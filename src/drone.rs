use crossbeam_channel::{select_biased, Receiver, Sender};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use wg_2024::controller::DroneEvent::ControllerShortcut;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::{FloodRequest, Nack, NackType, NodeType, Packet, PacketType};
pub struct BoberDrone {
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    packet_recv: Receiver<Packet>,
    pdr: f32,
    received_flood_requests: HashSet<(u64, NodeId)>,
    is_crashing: bool,
}

impl Drone for BoberDrone {
    fn new(
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
    ) -> Self {
        let mut pdr = pdr;
        if pdr < 0.0 {
            println!("Pdr lower than 0.0 (value: {}). Setting it to  0.0", pdr);
            pdr = 0.0;
        }
        if pdr > 1.0 {
            println!("Pdr greater than 1.0 (value: {}). Setting it to  1.0", pdr);
            pdr = 1.0;
        }
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            pdr,
            packet_send,
            received_flood_requests: HashSet::new(),
            is_crashing: false,
        }
    }
    fn run(&mut self) -> () {
        loop {
            select_biased! {
                recv(self.controller_recv) -> command => {
                    if self.is_crashing {
                        break;
                    }
                    if let Ok(command) = command {
                        if let DroneCommand::Crash = command {
                            println!("drone {} crashed", self.id);
                            break;
                        }
                        self.handle_command(command);
                    }
                }
                recv(self.packet_recv) -> packet => {
                    if let Ok(packet) = packet {
                        if self.is_crashing {
                            self.handle_packet_while_crashing(packet);
                        } else {
                            self.handle_packet(packet);
                        }
                    }
                },
            }
        }
    }
}

impl BoberDrone {
    fn set_pdr(&mut self, pdr: f32) {
        let mut pdr = pdr;
        if pdr < 0.0 {
            println!("Pdr lower than 0.0 (value: {}). Setting it to  0.0", pdr);
            pdr = 0.0;
        }
        if pdr > 1.0 {
            println!("Pdr greater than 1.0 (value: {}). Setting it to  1.0", pdr);
            pdr = 1.0;
        } else {
            let previous_pdr = self.pdr;
            self.pdr = pdr;
            println!("Pdr has been changed from {} to {}", previous_pdr, self.pdr);
        }
    }
    fn handle_packet_while_crashing(&mut self, packet: Packet) {
        match &packet.pack_type {
            PacketType::FloodRequest(_) => {}
            _ => {
                if self.check_if_packet_is_sendable(&packet) {
                    match &packet.pack_type {
                        PacketType::Nack(_) => self.forward_packet(&packet, false),
                        PacketType::Ack(_) => self.forward_packet(&packet, false),
                        PacketType::MsgFragment(_) => {
                            self.send_error_nack(NackType::ErrorInRouting(self.id), &packet)
                        }
                        PacketType::FloodResponse(_) => self.forward_packet(&packet, false),
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        match &packet.pack_type {
            PacketType::FloodRequest(_) => self.manage_flood_request(&packet),
            _ => {
                if self.check_if_packet_is_sendable(&packet) {
                    self.forward_packet(&packet, false)
                }
            },
        }
    }

    fn handle_command(&mut self, command: DroneCommand) {
        match command {
            DroneCommand::AddSender(node_id, sender) => {
                println!("Adding Sender: {} from Drone: {}", node_id, self.id);
                self.add_channel(node_id, sender);
            }
            DroneCommand::SetPacketDropRate(pdr) => {
                self.set_pdr(pdr);
            }
            DroneCommand::Crash => {
                print!("Crashing Drone: {}", self.id);
                self.is_crashing = true;
            }
            DroneCommand::RemoveSender(node_id) => {
                println!("Remove Sender: {} from Drone: {}", node_id, self.id);
                self.remove_channel(&node_id);
            }
        }
    }

    fn add_channel(&mut self, id: NodeId, sender: Sender<Packet>) {
        if self.packet_send.insert(id, sender).is_some() {
            println!("Channel has been correctly added to the drone.");
        } else {
            println!("An error occurred while trying to add the channel to the drone.");
        }
    }

    fn remove_channel(&mut self, id: &NodeId) {
        if self.packet_send.remove(id).is_some() {
            println!("Channel has been correctly removed from the drone.");
        } else {
            println!("An error occurred while trying to remove the channel from the drone.");
        }
    }

    //Creates a NACK with the specified NackType and sends it back
    fn send_error_nack(&mut self, nack: NackType, packet: &Packet) {
        match &packet.pack_type {
            PacketType::FloodRequest(_) | PacketType::MsgFragment(_) => {
                let mut srh_nack = packet.routing_header.clone();
                srh_nack = srh_nack.sub_route(0..srh_nack.hop_index+1).unwrap();
                srh_nack.reverse();
                srh_nack.reset_hop_index();
                if NackType::UnexpectedRecipient(self.id) == nack{
                    srh_nack.hops[0] = self.id;
                }
                let nack = Nack {
                    fragment_index: packet.get_fragment_index(),
                    nack_type: nack,
                };
                let new_packet = Packet {
                    pack_type: PacketType::Nack(nack),
                    routing_header: srh_nack,
                    session_id: packet.session_id,
                };
                self.forward_packet(&new_packet, true);
                self.controller_send
                    .send(DroneEvent::PacketDropped(packet.clone()))
                    .unwrap();
            }
            _ => self
                .controller_send
                .send(ControllerShortcut(packet.clone()))
                .unwrap(),
        }
    }

    //Forwards the current packet to the next node
    fn forward_packet(&mut self, packet: &Packet, generated_in_node: bool) {
        let mut new_packet = packet.clone();
        //Avoids increasing the hop_index if the packet has been created in the drone
        let next_hop = new_packet.routing_header.next_hop().unwrap();
        new_packet.routing_header.increase_hop_index();
        let communication_channel = self.packet_send.get(&next_hop).unwrap();
        communication_channel.send(new_packet.clone()).unwrap();
        match packet.pack_type {
            PacketType::Nack(_) => {
                if !generated_in_node{
                    self.notify_sc_sent_packet(&new_packet);
                }
            },
            _ => {
                self.notify_sc_sent_packet(&new_packet);
            }
        }
    }

    fn notify_sc_sent_packet(&mut self, packet: &Packet) {
        let res = self.controller_send.send(DroneEvent::PacketSent(packet.clone())).ok();
        if res.is_none() {
            println!("WARNING: No connection to the Simulation Controller for Drone: {}", self.id);
        }
        else{
            println!("The message has been correctly sent to the simulation controller");
        }
    }
    //Manages packets sending following the protocol
    fn check_if_packet_is_sendable(&mut self, packet: &Packet) -> bool {
        //Checks if packet has been sent to the wrong drone
        if packet.routing_header.current_hop().unwrap() != self.id {
            self.send_error_nack(NackType::UnexpectedRecipient(self.id), &packet);
            return false
        }
        //Checks if packet destination is supposed to be this drone
        if packet.routing_header.is_last_hop() {
            self.send_error_nack(NackType::DestinationIsDrone, packet);
            return false
        }
        //Calculates the next hop
        let next_hop = packet.routing_header.next_hop().unwrap();
        //Checks if it can communicate with the next drone
        if !self.packet_send.contains_key(&next_hop) {
            //Sends a NACK for Routing Error if it can't reach the next drone
            self.send_error_nack(NackType::ErrorInRouting(next_hop), packet);
            return false
        }
        let communication_channel = self.packet_send.get(&next_hop);
        //Checks if the channel exists
        if communication_channel.is_none() {
            //Returns false if the communication channel doesn't exist
            self.send_error_nack(NackType::ErrorInRouting(next_hop), packet);
            return false
        }
        //Generates a random number between 0 and 100 and check if the packet should be dropped
        let num = rand::thread_rng().gen_range(0..101);
        let dropped = num <= ((self.pdr * 100f32) as i32);
        //Checks if the packet can be dropped
        return match &packet.pack_type {
            //Can drop the packet only if it's a message
            PacketType::MsgFragment(_) => {
                //Checks if the package should be dropped
                if dropped {
                    //Creates a NACK for the dropped packet
                    self.send_error_nack(NackType::Dropped, packet);
                    false
                } else {
                    //Can send the packet
                    true
                }
            }
            //Can't drop the packet if it's not a message
            _ => true
        };
    }

    fn send_flood_response(&mut self, packet: &Packet, flood_request: &FloodRequest) {
        let mut flood_request = flood_request.clone();
        if flood_request.path_trace.len() > 1 &&
            flood_request.path_trace.last().unwrap().0 == flood_request.path_trace.iter().rev().nth(1).unwrap().0 &&
            flood_request.path_trace.last().unwrap().0 == self.id{
            flood_request.path_trace.remove(flood_request.path_trace.len() - 1);
        }
        let response = flood_request.generate_response(packet.session_id+1);
        self.forward_packet(&response, false);
    }

    fn manage_flood_request(&mut self, packet: &Packet) {
        match &packet.pack_type {
            //Checks if it's actually managing a Flood Request
            PacketType::FloodRequest(flood_request) => {
                //Get Sender id
                let mut sender_id = 0;
                if flood_request.path_trace.last().is_some(){
                    sender_id = flood_request.path_trace.last().unwrap().0;
                } else {
                    println!("Empty PathTrace found in drone: {}", self.id);
                    return
                };
                let flood_identifier = (flood_request.flood_id, flood_request.initiator_id);
                //Checks if it has already received this flood
                if !self.received_flood_requests.insert(flood_identifier) {
                    //Sends a response and avoids a loop
                    self.send_flood_response(packet, &flood_request);
                    return;
                }
                if flood_request.path_trace.contains(&(self.id, NodeType::Drone)) {
                    //Sends a response and avoids a loop
                    self.send_flood_response(packet, &flood_request);
                    return;
                }
                let mut flood_request = flood_request.get_incremented(self.id, NodeType::Drone);
                //Checks if the drone has Friends that are not the sender
                if self.packet_send.is_empty() || (self.packet_send.contains_key(&sender_id) && self.packet_send.len() == 1){
                    //Send a Flood Response if lonely (poor lonely drone :(  )
                    self.send_flood_response(packet, &flood_request);
                    return;
                }
                //Forwards the Flood Request to all the Neighbor Nodes (So many friends :) )
                for (&node_id, sender) in &self.packet_send {
                    //It ignores the Sender (Poor node :(  )
                    if node_id != sender_id{
                        //Creates the new Flood Request

                        let next_packet: Packet = Packet {
                            pack_type: PacketType::FloodRequest(flood_request.clone()),
                            routing_header: packet.routing_header.clone(),
                            session_id: packet.session_id,
                        };
                        //send event to notify the SC
                        self.controller_send
                            .send(DroneEvent::PacketSent(next_packet.clone()))
                            .unwrap();
                        //Forwards the Flood Request
                        sender.send(next_packet).unwrap();
                    }
                }
            }
            _ => println!(
                "Drone {}, expected packet of type FloodRequest found else instead",
                self.id
            ),
        }
    }
}
