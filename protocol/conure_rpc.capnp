@0xb5f42f99a8f00d55;

struct SystemInfo {
  clientId   @0  :Text;   # Unique identifier for this client
  hostname   @1  :Text;   # Machine hostname
  osType     @2  :Text;   # Operating system type (Windows, Linux, macOS)
  osVersion  @3  :Text;   # Operating system version
  osArch     @4  :Text;   # System architecture (x86, x86_64, arm64, etc.)
  currentTime @5 :Int64; # Current system time (Unix timestamp)
  timeZone   @6  :Text;   # System timezone
  userName   @7  :Text;   # Current username
}

struct ShellData {
  data @0 :Data;  # Raw byte data
}

# Bidirectional shell interface (can be used by both client and server)
interface ShellSession {
  # Send input to the shell
  sendInput @0 (input :ShellData) -> ();
  
  # Receive output from the shell
  receiveOutput @1 (output :ShellData) -> ();
  
  # Terminate the shell session
  terminate @2 () -> ();
}

# Initial service that returns the ServerRPC to communicate with the client.
interface Gateway {
  struct ConnectionParameters {
    client @0 :ClientRpc;
    token  @1 :Text;
  }

  registerClient @0 (params :ConnectionParameters) -> (rpc: ServerRpc);
}

# Server RPC, interacted with by client.
interface ServerRpc {
  reportSystemInfo @0 (info :SystemInfo) -> ();
}

# Client RPC, interacted with by server.
interface ClientRpc {
  # Asyncronously request a reportSystemInfo call.
  requestSystemInfo @0 () -> ();

  # Start a shell on the client
  startShell @1 () -> (session :ShellSession);
}