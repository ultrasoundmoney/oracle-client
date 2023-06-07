# Ultrasound Oracle client
Code that implements the node / client software for validators that want to participate in the ultrasound oracle network.

# Prerequesites
1. Rust + cargo
2. [gofer](https://github.com/chronicleprotocol/oracle-suite/blob/master/cmd/gofer/README.md)

# Get started
1. Install Prerequesites
2. `cargo install`
3. Set `GOFER_CMD` env variable to the absolute path to your `gofer` executable
4. Run an instance of the [oracle-server](https://github.com/ultrasoundmoney/oracle-server) and set the `SERVER_URL` environment variable to the full url of the endpoint to post oracle messages. 
5. Run the client with `cargo run`


