# Cepheus Trader operator-management protocol.
@0xad2ccbbf543ddce1;

using Cxx = import "/capnp/c++.capnp";
$Cxx.namespace("ct::admin");

struct Envelope {
  protocolVersion @0 :UInt16;
  requestId @1 :UInt64;

  union {
    request @2 :Request;
    response @3 :Response;
    close @4 :Close;
  }
}

struct Request {
  # Stable across retries. Exactly 16 bytes.
  commandId @0 :Data;
  union {
    addBbs @1 :AddBbs;
    initializeUniverse @2 :InitializeUniverse;
    status @3 :Void;
    liveBackup @4 :LiveBackup;
  }
}

struct LiveBackup {
  # A simple server-side backup label, not a filesystem path.
  label @0 :Text;
}

struct AddBbs {
  name @0 :Text;
}

struct InitializeUniverse {
}

struct Response {
  commandId @0 :Data;
  committedSequence @1 :UInt64;

  union {
    bbsAdded @2 :BbsCredentials;
    error @3 :Error;
    universeInitialized @4 :UniverseInitialized;
    status @5 :ServerStatus;
    backupComplete @6 :BackupComplete;
  }
}

struct ServerStatus {
  committedSequence @0 :UInt64;
  gameSecond @1 :UInt64;
  queuedInputs @2 :UInt64;
  bbsCount @3 :UInt32;
  playerCount @4 :UInt64;
  systemCount @5 :UInt64;
  activeSessions @6 :UInt32;
  storageFormat @7 :UInt64;
}

struct BackupComplete {
  label @0 :Text;
  committedSequence @1 :UInt64;
  gameSecond @2 :UInt64;
}

struct BbsCredentials {
  bbsId @0 :UInt32;
  # Raw 32-byte TLS external PSK. This field must never be logged.
  psk @1 :Data;
}

struct UniverseInitialized {
  # Random identity for this incarnation of the game universe.
  universeId @0 :Data;
  polityCount @1 :UInt32;
  systemCount @2 :UInt32;
  worldCount @3 :UInt32;
}

enum ErrorCode {
  invalidRequest @0;
  internalFailure @1;
}

struct Error {
  code @0 :ErrorCode;
  message @1 :Text;
}

struct Close {
  reason @0 :Text;
}
