# Cepheus Trader League Coordinator protocol.
@0xdf8ec3a87624abc1;

using Cxx = import "/capnp/c++.capnp";
$Cxx.namespace("ct::league");

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
  # Stable across retries. Exactly 16 bytes for mutating commands.
  commandId @0 :Data;
  union {
    status @1 :Void;
    setName @2 :SetName;
    addBbs @3 :AddBbs;
    disableBbs @4 :SetBbsAccess;
    enableBbs @5 :SetBbsAccess;
  }
}

struct SetName { expectedRevision @0 :UInt64; name @1 :Text; }
struct AddBbs { name @0 :Text; }
struct SetBbsAccess {
  bbsId @0 :UInt32;
  expectedRevision @1 :UInt64;
  reason @2 :Text;
}

struct Response {
  commandId @0 :Data;
  committedSequence @1 :UInt64;
  union {
    status @2 :LeagueStatus;
    nameSet @3 :LeagueStatus;
    bbsAdded @4 :BbsCredentials;
    memberUpdated @5 :LeagueMember;
    stale @6 :LeagueStatus;
    error @7 :Error;
    staleMember @8 :LeagueMember;
  }
}

struct LeagueStatus {
  leagueId @0 :UInt32;
  name @1 :Text;
  revision @2 :UInt64;
  members @3 :List(LeagueMember);
}
struct LeagueMember {
  bbsId @0 :UInt32;
  bbsName @1 :Text;
  enabled @2 :Bool;
  reason @3 :Text;
  revision @4 :UInt64;
}
struct BbsCredentials { bbsId @0 :UInt32; psk @1 :Data; }
enum ErrorCode { invalidRequest @0; internalFailure @1; accessDenied @2; }
struct Error { code @0 :ErrorCode; message @1 :Text; }
enum CloseCode {
  unspecified @0;
  unsupportedVersion @1;
  invalidRequest @2;
  accessDenied @3;
  internalFailure @4;
  serverStopping @5;
}
struct Close { code @0 :CloseCode; message @1 :Text; }
