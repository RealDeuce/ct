# Cepheus Trader BBS-sysop protocol.
@0xcfe1b226374c8d95;

using Cxx = import "/capnp/c++.capnp";
$Cxx.namespace("ct::sysop");

struct Envelope {
  protocolVersion @0 :UInt16;
  requestId @1 :UInt64;

  union {
    request @2 :Request;
    response @3 :Response;
    close @4 :Close;
    clientHello @5 :ClientHello;
    serverHello @6 :ServerHello;
  }
}

struct ClientHello { languageTag @0 :Text; }
struct ServerHello { languageTag @0 :Text; }

struct LegacyV1Envelope {
  protocolVersion @0 :UInt16;
  requestId @1 :UInt64;
  union {
    request @2 :Void;
    response @3 :Void;
    close @4 :LegacyClose;
  }
}
struct LegacyClose { reason @0 :Text; }

struct Request {
  # Exactly 16 bytes. Stable across retries for SetConfiguration;
  # GetConfiguration is an observation and returns the current revision.
  commandId @0 :Data;

  union {
    getConfiguration @1 :Void;
    setConfiguration @2 :SetConfiguration;
    getPlayerAccess @3 :PlayerTarget;
    setPlayerAccess @4 :SetPlayerAccess;
    taxPlayer @5 :TaxPlayer;
    demotePlayer @6 :DemotePlayer;
  }
}

struct TaxPlayer {
  playerId @0 :UInt32;
  credits @1 :UInt64;
}

struct DemotePlayer {
  playerId @0 :UInt32;
  navalGradeIndex @1 :UInt8;
}

struct PlayerTarget {
  playerId @0 :UInt32;
}

enum PlayerAccessState {
  active @0;
  suspended @1;
  removed @2;
}

struct SetPlayerAccess {
  playerId @0 :UInt32;
  expectedRevision @1 :UInt64;
  state @2 :PlayerAccessState;
  reason @3 :Text;
}

struct SetConfiguration {
  expectedRevision @0 :UInt64;
  settings @1 :BbsSettings;
}

struct BbsSettings {
  bbsName @0 :Text;
  polityName @1 :Text;

  # 0 is completely trade-oriented; 100 is completely combat-oriented.
  tradeCombat @2 :UInt8;

  # 0 is completely chaotic; 100 is completely institutionally orderly.
  chaosOrder @3 :UInt8;
}

struct Response {
  commandId @0 :Data;
  committedSequence @1 :UInt64;

  union {
    configuration @2 :BbsConfiguration;
    error @3 :Error;
    playerAccess @4 :PlayerAccess;
    directiveIssued @5 :DirectiveIssued;
  }
}

struct DirectiveIssued {
  playerId @0 :UInt32;
  messageId @1 :UInt64;
  issuedSecond @2 :UInt64;
  union {
    taxCredits @3 :UInt64;
    navalGradeIndex @4 :UInt8;
  }
}

struct PlayerAccess {
  playerId @0 :UInt32;
  revision @1 :UInt64;
  state @2 :PlayerAccessState;
  reason @3 :Text;
}

struct BbsConfiguration {
  bbsId @0 :UInt32;
  revision @1 :UInt64;
  configured @2 :Bool;
  settings @3 :BbsSettings;
}

enum ErrorCode {
  invalidRequest @0;
  staleRevision @1;
  internalFailure @2;
  noEligibleSite @3;
  permanentlyRemoved @4;
}

struct Error {
  code @0 :ErrorCode;
  message @1 :Text;
  currentRevision @2 :UInt64;
}

enum CloseCode {
  unspecified @0;
  unsupportedVersion @1;
  malformedHello @2;
  unsupportedLanguage @3;
  accessDenied @4;
  invalidRequest @5;
  staleSession @6;
  serverStopping @7;
  sessionReplaced @8;
  internalFailure @9;
}

struct Close {
  code @0 :CloseCode;
  message @1 :Text;
  supportedLanguageTags @2 :List(Text);
}
