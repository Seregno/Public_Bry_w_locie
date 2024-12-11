# Public_Bry_w_locie

Welcome to the official repository for the **Bobry w locie** drone!  
Here, you'll find the drone's source code in `src/drone.rs` and a `tests` folder containing scripts to verify the correct functioning of the *bober drone*.

---

## What is our product?

We built our drone with the same care and precision as a beaver builds its dike, following these principles:

- **Minimal:** A streamlined and efficient design.  
- **Versatile:** Adaptable to various use cases.  
- **Natural:** Inspired by logical and straightforward solutions.

Our product adheres to the rules outlined in the protocol, ensuring compatibility with other drones and ease of integration into your project. We improved unclear aspects of the protocol with common sense and insights from fields like **Algorithms** and **Networking**.

---

## What does our drone provide?

Our drone offers the essential features needed to establish a network for the **AP 2024/2025** class:

1. **Packet Management**  
   - Manages packets by distinguishing flood requests from other types.  
   - Forwards non-flood packets to the next node, checking if they are fragments and deciding whether to drop them.  
   - Sends NACKs for errors using the `send_error_nack()` method, which takes a `&mut self` reference and the problematic packet with a NACK type indicating the error response.

2. **Flooding Protocol**  
   - Implements protocol-compliant flooding using a `HashSet` to track received flood requests.  
   - Related code and logic can be found in the `handle_flood_request()` method.

3. **Command Handling**  
   - Manages add/remove channel commands and `set_pdr` efficiently.  
   - Handles crashes with a `crashing` flag that indicates whether the drone is still operational or about to shut down (RIP bober).

4. **Logging**  
   - All actions and their results are printed to the terminal, giving clear visibility into the drone's behavior.

---

## Tests

The `tests` folder contains scripts to ensure the drone works as expected. These tests cover all features and are categorized based on different event types the drone may encounter. Special focus has been given to testing **flooding**, **NACKs**, and **forwarding**, given the drone's interactions with a wide network.

---

## Channel Support

- **Telegram Group for Customer Support:**  
  [Join here](https://t.me/+HVC865P-e3ZmZDJk)

> **Note:** Contact us only if necessary. Please, no AI-generated "Good Morning" pictures. 😉

---

Enjoy building with **Bobry w locie**!
