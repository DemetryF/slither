A simple clone of the [slither.io](http://slither.com/io) game. Made for tokio learning.

# Starting
To start the server and write:
```sh
cargo run --bin backend
```
After the server is succesfully started you will see the port. 
You should use it to connect to the server. 
Also you can specify it by your own:
```sh
cargo run --bin backend -- --port 8080
```

To start the client write:
```sh
cargo run --bin frontend
```
