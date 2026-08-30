# Shared CT-RPC wire schema. Both the Rust server and C++ client generate their
# language bindings from this file.
@0xe8b8f4da93b0126e;

using Cxx = import "/capnp/c++.capnp";
$Cxx.namespace("ct::rpc");

enum Phase {
  disconnected @0;
  jump @1;
  interplanetary @2;
  encounter @3;
  docked @4;
  onPlanet @5;
  newUser @6;
  terminal @7;
}

struct Envelope {
  protocolVersion @0 :UInt16;
  requestId @1 :UInt64;
  sessionEpoch @2 :UInt64;

  union {
    clientHello @3 :ClientHello;
    serverHello @4 :ServerHello;
    request @5 :Request;
    response @6 :Response;
    event @7 :Event;
    cancel @8 :Cancel;
    close @9 :Close;
  }
}

struct ClientHello {
  identity @0 :PlayerIdentity;
  clientName @1 :Text;
  languageTag @2 :Text;
}

struct ServerHello {
  identity @0 :PlayerIdentity;
  assignedEpoch @1 :UInt64;
  committedSequence @2 :UInt64;
  phase @3 :Phase;
  languageTag @4 :Text;
  formatting @5 :DisplayFormatting;
  affiliation @6 :InstitutionalAffiliation;
  accountJournalAvailable @7 :Bool;
}

struct InstitutionalAffiliation {
  polityName @0 :Text;
  bbsName @1 :Text;
  # Empty when the home BBS is independent.
  leagueName @2 :Text;
}

struct DisplayFormatting {
  decimalSeparator @0 :Text;
  groupingSeparator @1 :Text;
  primaryGroupingDigits @2 :UInt8;
  secondaryGroupingDigits @3 :UInt8;
  gameTimestampPattern @4 :Text;
  gameDurationPattern @5 :Text;
  realDurationPattern @6 :Text;
}

# This wire layout originated in CT-RPC 2 and is now used only to send an
# actionable rejection to obsolete clients. Established sessions always use
# Envelope above.
struct LegacyV2Envelope {
  protocolVersion @0 :UInt16;
  requestId @1 :UInt64;
  sessionEpoch @2 :UInt64;
  union {
    clientHello @3 :Void;
    serverHello @4 :Void;
    request @5 :Void;
    response @6 :Void;
    event @7 :Void;
    cancel @8 :Void;
    close @9 :LegacyClose;
  }
}

struct LegacyClose { reason @0 :Text; }

struct PlayerIdentity {
  # The server-assigned BBS ID is encoded as canonical unsigned decimal text
  # when used as the TLS external-PSK identity. The player ID is local to that
  # authenticated BBS.
  bbsId @0 :UInt32;
  playerId @1 :UInt32;
}

struct Request {
  # Exactly 16 bytes. For state-changing requests this is stable across
  # retries and reconnects and provides exactly-once replay. For observations
  # it remains a correlation identifier; the server returns a fresh ordered
  # view and does not retain the response indefinitely.
  commandId @0 :Data;

  union {
    ping @1 :Void;
    createPlayer @2 :PlayerCreation;
    getCaptainCreationOptions @3 :Void;
    getStartingShipOffers @4 :Void;
    getStartingShipOptions @5 :StartingShipOptionsRequest;
    getStartingCrewPlan @6 :StartingCrewPlanRequest;
    getCrewManagement @7 :Void;
    setCrewTrainingTarget @8 :SetCrewTrainingTargetRequest;
    setCrewAssignments @9 :SetCrewAssignmentsRequest;
    getShipStatus @10 :Void;
    getDockedSnapshot @11 :Void;
    getKnownDestinations @12 :Void;
    getMarket @13 :Void;
    buyCargo @14 :BuyCargoRequest;
    sellCargo @15 :SellCargoRequest;
    getTravelStatus @16 :Void;
    beginVoyage @17 :BeginVoyageRequest;
    plotCourse @18 :PlotCourseRequest;
    openArrivalPacket @19 :Void;
    getMessageManagement @20 :Void;
    setMessageClassification @21 :SetMessageClassificationRequest;
    setSystemMappingDisclosure @22 :SetSystemMappingDisclosureRequest;
    getFlightPlan @23 :Void;
    previewFlightPlan @24 :FlightPlanProposal;
    commitFlightPlan @25 :CommitFlightPlanRequest;
    acknowledgeCheckpoint @26 :AcknowledgeCheckpointRequest;
    getEncounter @27 :Void;
    resolveEncounter @28 :ResolveEncounterRequest;
    getTaskLedger @29 :Void;
    acceptTaskOffer @30 :AcceptTaskOfferRequest;
    setCarriageDeclaration @31 :CarriageDeclarationRequest;
    getFinance @32 :Void;
    getMarketKnowledge @33 :Void;
    getShipMarket @34 :Void;
    purchaseShip @35 :PurchaseShipRequest;
    getCrewMarket @36 :Void;
    hireCrew @37 :HireCrewRequest;
    beginMarketSearch @38 :BeginMarketSearchRequest;
    cancelWorkAssignment @39 :CancelWorkAssignmentRequest;
    getCombat @40 :Void;
    submitCombatOrder @41 :CombatOrderSet;
    setCombatAutomationPolicy @42 :CombatAutomationPolicy;
    getCombatCareer @43 :Void;
    acceptCareerOpportunity @44 :CareerOpportunityRequest;
    engageTrafficContact @45 :EngageTrafficContactRequest;
    setPirateCruise @46 :PirateCruiseRequest;
    settlePrize @47 :PrizeSettlementRequest;
    settleWarrant @48 :WarrantSettlementRequest;
    setMessageFilter @49 :SetMessageFilterRequest;
    getDockedServices @50 :Void;
    commitDockedService @51 :DockedServiceOrder;
    reserveMarketLead @52 :ReserveMarketLeadRequest;
    releaseMarketReservation @53 :ReleaseMarketReservationRequest;
    applyTaskAction @54 :TaskActionRequest;
    sendPrivateMessage @55 :SendPrivateMessageRequest;
    purchaseInsurance @56 :PurchaseInsuranceRequest;
    getFleet @57 :Void;
    setActiveShip @58 :SetActiveShipRequest;
    assignShipCaptain @59 :AssignShipCaptainRequest;
    transferShipStores @60 :TransferShipStoresRequest;
    setCombatCareerMode @61 :CombatCareerModeRequest;
    recoverCommand @62 :RecoverCommandRequest;
    declareBankruptcy @63 :RecoverCommandRequest;
    applyPersonnelAction @64 :PersonnelActionRequest;
    getSystemRadio @65 :Void;
    transmitSystemRadio @66 :TransmitSystemRadioRequest;
    peekRadioReception @67 :RadioReceptionRequest;
    acknowledgeRadioReception @68 :RadioReceptionRequest;
    setRadioMute @69 :SetRadioMuteRequest;
    misappropriateRestrictedCredits @70 :MisappropriateRestrictedCreditsRequest;
    setInterceptionWatch @71 :InterceptionWatchRequest;
    abandonPlayer @72 :AbandonPlayerRequest;
    suggestTaskCourse @73 :Void;
    cureFinanceDefault @74 :Void;
    commissionShip @75 :CommissionShipRequest;
    getTerminalReport @76 :Void;
    beginMarketNegotiation @86 :BeginMarketNegotiationRequest;
    acceptMarketQuote @87 :MarketQuoteActionRequest;
    rejectMarketQuote @88 :MarketQuoteActionRequest;
    acknowledgeTerminalReport @77 :AcknowledgeTerminalReportRequest;
    getBrowserAlertStatus @78 :Void;
    createBrowserAlertEnrollment @79 :Void;
    revokeAllBrowserAlerts @80 :Void;
    getOperationalDamageReport @81 :Void;
    acknowledgeOperationalDamageReport @82 :AcknowledgeOperationalDamageReportRequest;
    getEncounterPolicyDefault @83 :Void;
    setEncounterPolicyDefault @84 :SetEncounterPolicyDefaultRequest;
    getAccountLedger @85 :AccountLedgerRequest;
  }
}

struct Response {
  commandId @0 :Data;
  committedSequence @1 :UInt64;
  revision @2 :UInt64;
  phase @3 :Phase;
  replayed @4 :Bool;

  union {
    pong @5 :Void;
    error @6 :Error;
    playerCreated @7 :PlayerCreation;
    captainCreationOptions @8 :CaptainCreationOptions;
    startingShipOffers @9 :StartingShipOffers;
    startingShipOptions @10 :StartingShipOptions;
    startingCrewPlan @11 :StartingCrewPlan;
    crewManagement @12 :CrewManagementSnapshot;
    shipStatus @13 :ShipStatusSnapshot;
    dockedSnapshot @14 :DockedSnapshot;
    knownDestinations @15 :KnownDestinations;
    market @16 :MarketSnapshot;
    travelStatus @17 :TravelStatus;
    coursePlot @18 :CoursePlot;
    arrivalPacket @19 :ArrivalPacket;
    messageManagement @20 :MessageManagement;
    systemMappingStatus @21 :SystemMappingStatus;
    flightPlan @22 :FlightPlanSnapshot;
    flightPlanPreview @23 :FlightPlanPreview;
    checkpoint @24 :CheckpointSnapshot;
    encounter @25 :EncounterSnapshot;
    encounterResult @26 :EncounterResult;
    taskLedger @27 :TaskLedger;
    finance @28 :FinanceSnapshot;
    marketKnowledge @29 :MarketKnowledge;
    shipMarket @30 :ShipMarket;
    crewMarket @31 :CrewMarket;
    combat @32 :CombatSnapshot;
    combatCareer @33 :CombatCareerSnapshot;
    dockedServices @34 :DockedServices;
    fleet @35 :FleetSnapshot;
    systemRadio @36 :SystemRadioSnapshot;
    radioContent @37 :RadioContent;
    dockedServiceReceipt @38 :DockedServiceReceipt;
    terminalReport @39 :TerminalReport;
    browserAlertStatus @40 :BrowserAlertStatus;
    browserAlertEnrollment @41 :BrowserAlertEnrollment;
    operationalDamageReport @42 :OperationalDamageReport;
    encounterPolicyDefault @43 :EncounterPolicyDefaultSnapshot;
    accountLedger @44 :AccountLedgerPage;
  }
}

struct BrowserAlertStatus {
  configured @0 :Bool;
  activeDevices @1 :UInt32;
  maximumDevices @2 :UInt32;
}

struct BrowserAlertEnrollment {
  status @0 :BrowserAlertStatus;
  url @1 :Text;
  expiresUnixSecond @2 :UInt64;
}

struct SkillPool {
  level3 @0 :UInt8;
  level2 @1 :UInt8;
  level1 @2 :UInt8;
  level0 @3 :UInt8;
}

struct CharacteristicPointBuy {
  minimum @0 :UInt8;
  maximum @1 :UInt8;
  neutral @2 :UInt8;
  budget @3 :Int16;
}

enum SkillId {
  admin @0;
  advocate @1;
  astrogation @2;
  broker @3;
  carouse @4;
  communications @5;
  computer @6;
  electronics @7;
  engineerJump @8;
  engineerManeuver @9;
  engineerPower @10;
  engineerLifeSupport @11;
  etiquette @12;
  gunCombat @13;
  gunnerTurrets @14;
  gunnerCapital @15;
  gunnerScreens @16;
  investigate @17;
  jackOfAllTrades @18;
  leadership @19;
  mechanic @20;
  medicine @21;
  melee @22;
  persuade @23;
  pilotSpacecraft @24;
  pilotSmallCraft @25;
  recon @26;
  stealth @27;
  streetwise @28;
  tacticsMilitary @29;
  tacticsNaval @30;
  tradeCargomaster @31;
  vaccSuit @32;
  tradeProspector @33;
}

struct SkillDefinition {
  id @0 :SkillId;
  name @1 :Text;
}

struct SkillRating {
  skill @0 :SkillId;
  level @1 :Int8;
}

struct SkillTraining {
  skill @0 :SkillId;
  neededWeeks @1 :UInt16;
  currentWeeks @2 :UInt16;
}

struct Characteristics {
  strength @0 :UInt8;
  dexterity @1 :UInt8;
  endurance @2 :UInt8;
  intelligence @3 :UInt8;
  education @4 :UInt8;
  charisma @5 :UInt8;
}

struct PersonDraft {
  name @0 :Text;
  characteristics @1 :Characteristics;
  skills @2 :List(SkillRating);
  training @3 :SkillTraining;
}

struct InitialCrewDraft {
  slotId @0 :UInt16;
  name @1 :Text;
  trainingSkill @2 :SkillId;
}

struct PlayerCreation {
  setupRevision @0 :UInt64;
  startingOfferId @1 :UInt32;
  captain @2 :PersonDraft;
  shipName @3 :Text;
  crew @4 :List(InitialCrewDraft);
  refitOptionIds @5 :List(UInt32);
}

struct CaptainCreationOptions {
  setupRevision @0 :UInt64;
  skillPool @1 :SkillPool;
  permittedSkills @2 :List(SkillDefinition);
  defaultCaptain @3 :PersonDraft;
  characteristicPointBuy @4 :CharacteristicPointBuy;
}

enum Career {
  trader @0;
  privateer @1;
  navy @2;
}

struct OriginDossier {
  bbsName @0 :Text;
  polityName @1 :Text;
  homeSystemName @2 :Text;
  homeWorldName @3 :Text;
  tradeCombat @4 :UInt8;
  chaosOrder @5 :UInt8;
  # Empty when the originating BBS is independent.
  leagueName @6 :Text;
}

struct StartingShipOfferSummary {
  offerId @0 :UInt32;
  career @1 :Career;
  packageName @2 :Text;
  shipCatalogId @3 :UInt32;
  shipName @4 :Text;
  role @5 :Text;
  rationale @6 :Text;
  displacementTons @7 :UInt32;
  jumpRating @8 :UInt8;
  thrustG @9 :UInt8;
  cargoTons @10 :Float64;
  crewCount @11 :UInt16;
  priceCredits @12 :UInt64;
}

struct StartingShipOffers {
  setupRevision @0 :UInt64;
  origin @1 :OriginDossier;
  offers @2 :List(StartingShipOfferSummary);
}

struct StartingShipOptionsRequest {
  setupRevision @0 :UInt64;
  startingOfferId @1 :UInt32;
}

struct StartingShipOptions {
  setupRevision @0 :UInt64;
  offer @1 :StartingShipOfferSummary;
  descriptionParagraphs @2 :List(Text);
  terms @3 :StartingOfferTerms;
  refitGroups @4 :List(StartingRefitGroup);
}

enum StartingTitleKind { ownedWithLien @0; sponsorOwned @1; institutionOwned @2; }
struct StartingOfferTerms {
  termsRevision @0 :UInt64;
  title @1 :StartingTitleKind;
  equityCredits @2 :UInt64;
  principalCredits @3 :UInt64;
  monthlyPaymentCredits @4 :UInt64;
  liquidReserveCredits @5 :UInt64;
  restrictedReserveCredits @6 :UInt64;
  monthlyCompensationCredits @7 :UInt64;
  refitCreditLimit @8 :UInt64;
  refitDisplacementMillitons @9 :UInt64;
  authority @10 :Text;
  exitTerms @11 :Text;
  insurance @12 :Text;
}
struct StartingRefitOption {
  optionId @0 :UInt32;
  name @1 :Text;
  description @2 :Text;
  displacementDeltaMillitons @3 :Int64;
  priceDeltaCredits @4 :Int64;
}
struct StartingRefitGroup {
  groupId @0 :UInt16;
  name @1 :Text;
  required @2 :Bool;
  options @3 :List(StartingRefitOption);
}

struct StartingCrewPlanRequest {
  setupRevision @0 :UInt64;
  startingOfferId @1 :UInt32;
}

struct StartingCrewSlot {
  slotId @0 :UInt16;
  role @1 :Text;
  representedPositions @2 :UInt16;
  required @3 :Bool;
  skillPool @4 :SkillPool;
  defaultCrew @5 :PersonDraft;
  roleKind @6 :CrewRoleKind;
}

struct StartingCrewPlan {
  setupRevision @0 :UInt64;
  startingOfferId @1 :UInt32;
  slots @2 :List(StartingCrewSlot);
}

struct SetCrewTrainingTargetRequest {
  personId @0 :UInt64;
  skill @1 :SkillId;
}

struct SetCrewAssignmentsRequest {
  personId @0 :UInt64;
  slotIds @1 :List(UInt16);
}

enum PersonnelActionKind {
  dismiss @0;
  transfer @1;
  shoreLeave @2;
  recall @3;
  firstAid @4;
  surgery @5;
  medicalCare @6;
}

struct PersonnelActionRequest {
  personId @0 :UInt64;
  expectedServiceRevision @1 :UInt64;
  action @2 :PersonnelActionKind;
  targetShipId @3 :UInt64;
  durationDays @4 :UInt16;
}

struct CrewRole {
  slotId @0 :UInt16;
  role @1 :Text;
  representedPositions @2 :UInt16;
  roleKind @3 :CrewRoleKind;
}

enum CrewRoleKind {
  command @0;
  pilot @1;
  navigator @2;
  engineer @3;
  sensorsOperator @4;
  screenOperator @5;
  turretGunner @6;
  bayGunner @7;
  gunner @8;
  medic @9;
  marine @10;
  flightCrew @11;
  steward @12;
  other @13;
}

struct CrewManagementMember {
  personId @0 :UInt64;
  slotId @1 :UInt16;
  role @2 :Text;
  representedPositions @3 :UInt16;
  captain @4 :Bool;
  person @5 :PersonDraft;
  assignedSlotIds @6 :List(UInt16);
  condition @7 :PersonCondition;
  injuryPoints @8 :UInt16;
  fatiguePoints @9 :UInt16;
  available @10 :Bool;
  currentStrength @11 :UInt8;
  currentDexterity @12 :UInt8;
  currentEndurance @13 :UInt8;
  serviceKind @14 :CrewServiceKind;
  monthlySalaryCredits @15 :UInt64;
  arrearsCredits @16 :UInt64;
  prizeShareBasisPoints @17 :UInt16;
  morale @18 :UInt8;
  loyalty @19 :UInt8;
  riskTolerance @20 :UInt8;
  availability @21 :CrewAvailability;
  availableSecond @22 :UInt64;
  serviceRevision @23 :UInt64;
  shoreLocation @24 :Text;
  roleKind @25 :CrewRoleKind;
  locationKind @26 :CrewLocationKind;
  unfedDays @27 :UInt16;
}

enum CrewLocationKind { aboardShip @0; shoreFacility @1; }

enum CrewServiceKind { ownerCaptain @0; salaried @1; prizeShare @2; institutional @3; }
enum CrewAvailability { active @0; shoreLeave @1; medicalCare @2; detached @3; awaitingRecall @4; }

enum PersonCondition {
  fit @0;
  fatigued @1;
  wounded @2;
  incapacitated @3;
  dead @4;
}

struct CrewManagementSnapshot {
  shipId @0 :UInt64;
  shipName @1 :Text;
  members @2 :List(CrewManagementMember);
  roles @3 :List(CrewRole);
  establishedComplement @4 :UInt16;
}

enum ShipSubsystemKind {
  hull @0;
  structure @1;
  armor @2;
  bridge @3;
  computer @4;
  sensors @5;
  jumpDrive @6;
  maneuverDrive @7;
  powerPlant @8;
  fuelSystem @9;
  lifeSupport @10;
  cargoHold @11;
  weaponMount @12;
  screen @13;
  hangar @14;
  other @15;
}

struct ShipSubsystemStatus {
  subsystemId @0 :UInt16;
  kind @1 :ShipSubsystemKind;
  label @2 :Text;
  maximumHits @3 :UInt16;
  sustainedHits @4 :UInt16;
  battlefieldRepairHits @5 :UInt16;
  effectiveHits @6 :UInt16;
  operationalEffect @7 :Text;
  lastProperRepairSecond @8 :UInt64;
  installedSecond @9 :UInt64;
  lastRefitSecond @10 :UInt64;
  calendarAgeMonths @11 :UInt32;
  operatingSeconds @12 :UInt64;
  dutyCycles @13 :UInt32;
  skimmingCycles @14 :UInt32;
  neglectDamageHits @15 :UInt16;
  displacementMillitons @16 :UInt64;
  replacementPriceCredits @17 :UInt64;
  installationGeneration @18 :UInt16;
  reconditioned @19 :Bool;
}

struct ShipAmmunitionStatus {
  ammunitionId @0 :Text;
  remaining @1 :UInt32;
  capacity @2 :UInt32;
  packUnits @3 :UInt32;
  pricePerPackCredits @4 :UInt64;
}

struct ShipProvisionStatus {
  personDaysRemaining @0 :UInt64;
  capacityPersonDays @1 :UInt64;
}

struct ShipStatusSnapshot {
  shipId @0 :UInt64;
  shipName @1 :Text;
  catalogId @2 :UInt32;
  catalogRevision @3 :UInt64;
  systemId @4 :UInt64;
  currentGameSecond @5 :UInt64;
  displacementMillitons @6 :UInt64;
  jumpRating @7 :UInt8;
  thrustG @8 :UInt8;
  fuelCapacityMillitons @9 :UInt64;
  currentFuelMillitons @10 :UInt64;
  jumpFuelMillitons @11 :UInt64;
  cargoCapacityMillitons @12 :UInt64;
  subsystems @13 :List(ShipSubsystemStatus);
  monthlyMaintenanceCredits @14 :UInt64;
  nextMaintenanceSecond @15 :UInt64;
  maintenancePaidThroughSecond @16 :UInt64;
  # Retired monetary-arrears slot; servers send zero for wire compatibility.
  maintenanceArrearsCredits @17 :UInt64;
  completedMaintenanceCycles @18 :UInt32;
  consecutiveMissedMaintenance @19 :UInt16;
  commissionedSecond @20 :UInt64;
  transitCount @21 :UInt32;
  warrantyExpiresSecond @22 :UInt64;
  warrantyTransitLimit @23 :UInt32;
  warrantyRepairs @24 :UInt16;
  lastRefitSecond @25 :UInt64;
  completedRefits @26 :UInt16;
  activeActivity @27 :ShipActivityStatus;
  unrefinedFuelMillitons @28 :UInt64;
  warrantyVoided @29 :Bool;
  monthlyLifeSupportCredits @30 :UInt64;
  recoveryStatus @31 :Text;
  shipRevision @32 :UInt64;
  ammunition @33 :List(ShipAmmunitionStatus);
  provisions @34 :ShipProvisionStatus;
  manifestedSymptoms @35 :List(Text);
  clockRateGameSeconds @36 :UInt64;
  clockRateRealSeconds @37 :UInt64;
}

enum DockedFuelServiceKind {
  refined @0;
  unrefined @1;
  gasGiant @2;
  wildernessWater @3;
}

enum FuelSourceBodyKind {
  notApplicable @0;
  gasGiant @1;
  planet @2;
  moon @3;
  icyBelt @4;
}

enum FuelAccessKind {
  portSale @0;
  routineWilderness @1;
}

struct DockedFuelService {
  kind @0 :DockedFuelServiceKind;
  label @1 :Text;
  sourceBodyId @2 :UInt32;
  hasSourceBody @3 :Bool;
  available @4 :Bool;
  unavailableReason @5 :Text;
  pricePerTonCredits @6 :UInt64;
  maximumMillitons @7 :UInt64;
  serviceSeconds @8 :UInt64;
  bodyKind @9 :FuelSourceBodyKind;
  accessKind @10 :FuelAccessKind;
  canRefine @11 :Bool;
  roundTripDistanceMicroAu @12 :UInt64;
  roundTripSeconds @13 :UInt64;
}

struct DockedRepairService {
  subsystemId @0 :UInt16;
  label @1 :Text;
  available @2 :Bool;
  unavailableReason @3 :Text;
  costCredits @4 :UInt64;
  serviceSeconds @5 :UInt64;
  replacement @6 :Bool;
  reconditioned @7 :Bool;
}

struct DockedServices {
  shipRevision @0 :UInt64;
  currentGameSecond @1 :UInt64;
  fuel @2 :List(DockedFuelService);
  ammunition @3 :List(ShipAmmunitionStatus);
  provisions @4 :ShipProvisionStatus;
  provisionPackagePersonDays @5 :UInt64;
  provisionPackagePriceCredits @6 :UInt64;
  repair @7 :List(DockedRepairService);
  refitAvailable @8 :Bool;
  refitUnavailableReason @9 :Text;
  refitCostCredits @10 :UInt64;
  refitServiceSeconds @11 :UInt64;
  provisionsAvailable @12 :Bool;
  ammunitionAvailable @13 :Bool;
}

struct DockedFuelOrder {
  kind @0 :DockedFuelServiceKind;
  sourceBodyId @1 :UInt32;
  hasSourceBody @2 :Bool;
  quantityMillitons @3 :UInt64;
}

struct DockedAmmunitionOrder {
  ammunitionId @0 :Text;
  packs @1 :UInt32;
}

struct DockedRepairOrder {
  subsystemId @0 :UInt16;
  reconditioned @1 :Bool;
}

struct DockedServiceOrder {
  expectedShipRevision @0 :UInt64;
  union {
    fuel @1 :DockedFuelOrder;
    ammunition @2 :DockedAmmunitionOrder;
    provisions @3 :UInt16;       # installed monthly packages
    properRepair @4 :UInt16;     # subsystem ID
    refit @5 :Void;
    replacement @6 :DockedRepairOrder;
  }
}

struct FuelPurchaseReceipt {
  kind @0 :DockedFuelServiceKind;
  quantityMillitons @1 :UInt64;
  currentFuelMillitons @2 :UInt64;
  unrefinedFuelMillitons @3 :UInt64;
  fuelCapacityMillitons @4 :UInt64;
  costCredits @5 :UInt64;
  restrictedPaymentCredits @6 :UInt64;
  liquidPaymentCredits @7 :UInt64;
  restrictedBalanceCredits @8 :UInt64;
  liquidBalanceCredits @9 :UInt64;
}

struct ProvisionPurchaseReceipt {
  packages @0 :UInt16;
  personDaysLoaded @1 :UInt64;
  personDaysRemaining @2 :UInt64;
  capacityPersonDays @3 :UInt64;
  costCredits @4 :UInt64;
  restrictedPaymentCredits @5 :UInt64;
  liquidPaymentCredits @6 :UInt64;
  restrictedBalanceCredits @7 :UInt64;
  liquidBalanceCredits @8 :UInt64;
}

struct DockedServiceReceipt {
  shipStatus @0 :ShipStatusSnapshot;
  detail :union {
    generic @1 :Void;
    fuelPurchase @2 :FuelPurchaseReceipt;
    provisionPurchase @3 :ProvisionPurchaseReceipt;
  }
}

struct ShipActivityStatus {
  activityId @0 :UInt64;
  startedSecond @1 :UInt64;
  dueSecond @2 :UInt64;
  costCredits @3 :UInt64;
  sourceBodyId @4 :UInt32;
  hasSourceBody @10 :Bool;

  union {
    none @5 :Void;
    refit @6 :Void;
    properRepair @7 :UInt16;       # subsystem ID
    gasGiantSkim @8 :UInt64;      # quantity in millitons
    wildernessWater @9 :UInt64;   # quantity in millitons
    refurbishment @11 :UInt16;    # number of replaced/reconditioned components
    escortDuty @12 :UInt64;       # operations-order identifier
    fieldRecovery @13 :UInt16;    # subsystem ID under crew field repair
    construction @14 :Void;
    fuelProcessing @15 :UInt64;    # quantity in millitons
  }
  refineCollected @16 :Bool;
}

struct DockedSnapshot {
  shipId @0 :UInt64;
  shipName @1 :Text;
  systemId @2 :UInt64;
  systemName @3 :Text;
  worldId @4 :UInt64;
  worldName @5 :Text;
  facilityId @6 :UInt64;
  facilityName @7 :Text;
  starport @8 :Text;
  techLevel @9 :UInt8;
  population @10 :UInt8;
  lawLevel @11 :UInt8;
  arrivedSecond @12 :UInt64;
  credits @13 :UInt64;
  debtCredits @14 :UInt64;
  fuelMillitons @15 :UInt64;
  fuelCapacityMillitons @16 :UInt64;
  refinedFuelPricePerTon @17 :UInt64;
  cargoUsedMillitons @18 :UInt64;
  cargoCapacityMillitons @19 :UInt64;
  unrefinedFuelMillitons @20 :UInt64;
  unrefinedFuelPricePerTon @21 :UInt64;
  accruedBerthFeeCredits @22 :UInt64;
  facilityRevision @23 :UInt64;
  personnelAvailable @24 :Bool;
  bankingAvailable @25 :Bool;
  authorityAvailable @26 :Bool;
  medicalLevel @27 :UInt8;
  clearanceRequired @28 :Bool;
  restrictedCredits @29 :UInt64;
  currentGameSecond @30 :UInt64;
}

struct KnownSystemSummary {
  systemId @0 :UInt64;
  systemName @1 :Text;
  worldName @2 :Text;
  distanceParsecs @3 :Float64;
  withinJumpRating @4 :Bool;
  starport @5 :Text;
  population @6 :UInt8;
  techLevel @7 :UInt8;
  observedSecond @8 :UInt64;
  source @9 :Text;
  corewardParsecs @10 :Float64;
  spinwardParsecs @11 :Float64;
  northParsecs @12 :Float64;
  remoteCandidate @13 :Bool;
  knowledgeSource @14 :SystemKnowledgeSource;
  gasGiantCount @15 :UInt8;
  # Present only when this system's institutional mapping is public.
  affiliation @16 :InstitutionalAffiliation;
  navigationTargets @17 :List(InSystemNavigationTarget);
}

enum InSystemNavigationTargetKind {
  rockyBody @0;
  gasGiant @1;
  planetoidBelt @2;
}

struct InSystemNavigationTarget {
  bodyId @0 :UInt32;
  name @1 :Text;
  kind @2 :InSystemNavigationTargetKind;
  primaryWorld @3 :Bool;
}

enum SystemKnowledgeSource {
  publishedRecords @0;
  carriedRecords @1;
  privateObservation @2;
  publicDispatch @3;
  directDispatch @4;
  withheld @5;
  secretChart @6;
}

struct KnownDestinations {
  currentSystemId @0 :UInt64;
  jumpRating @1 :UInt8;
  systems @2 :List(KnownSystemSummary);
  belts @3 :List(KnownBelt);
}

struct KnownBelt {
  systemId @0 :UInt64;
  bodyId @1 :UInt32;
  name @2 :Text;
  icy @3 :Bool;
  carbonaceousPercent @4 :UInt8;
  silicateOrRockPercent @5 :UInt8;
  metalOrWaterIcePercent @6 :UInt8;
  hydrocarbonPercent @7 :UInt8;
}

struct PlotCourseRequest {
  originSystemId @0 :UInt64;
  destinationSystemId @1 :UInt64;
  # Current-location plots may use fuel already aboard. Arbitrary
  # primary-to-primary comparisons begin with a normal refueling choice.
  useCurrentFuel @2 :Bool;
}

enum CourseFuelSource {
  none @0;
  carried @1;
  refinedPort @2;
  frontierSkimming @3;
  unrefinedPort @4;
}

struct CourseWaypoint {
  systemId @0 :UInt64;
  systemName @1 :Text;
  worldName @2 :Text;
  fuelSource @3 :CourseFuelSource;
  nextLegMilliparsecs @4 :UInt64;
}

struct CoursePlan {
  available @0 :Bool;
  elapsedSeconds @1 :UInt64;
  fuelCostCredits @2 :UInt64;
  totalMilliparsecs @3 :UInt64;
  waypoints @4 :List(CourseWaypoint);
}

struct CoursePlot {
  originSystemId @0 :UInt64;
  destinationSystemId @1 :UInt64;
  jumpRating @2 :UInt8;
  fastest @3 :CoursePlan;
  cheapest @4 :CoursePlan;
  currentGameSecond @5 :UInt64;
  clockRateGameSeconds @6 :UInt64;
  clockRateRealSeconds @7 :UInt64;
}

struct PriceDistribution {
  minimum @0 :UInt64;
  lowerQuartile @1 :UInt64;
  median @2 :UInt64;
  upperQuartile @3 :UInt64;
  maximum @4 :UInt64;
}

struct MarketOffer {
  offerId @0 :UInt64;
  commodityId @1 :UInt16;
  commodityName @2 :Text;
  basePricePerTon @3 :UInt64;
  purchasePricePerTon @4 :UInt64;
  salePricePerTon @5 :UInt64;
  availableMillitons @6 :UInt64;
  legality @7 :CommodityLegality;
  priceDistribution @8 :PriceDistribution;
}

enum CommodityLegality {
  legal @0;
  restricted @1;
  prohibited @2;
}

enum CargoTitle {
  playerOwned @0;
  freight @1;
  contract @2;
  uniqueObject @3;
}

struct CargoLot {
  cargoLotId @0 :UInt64;
  commodityId @1 :UInt16;
  commodityName @2 :Text;
  quantityMillitons @3 :UInt64;
  purchasePricePerTon @4 :UInt64;
  originSystemId @5 :UInt64;
  acquiredSecond @6 :UInt64;
  title @7 :CargoTitle;
  taskId @8 :UInt64;
  uniqueObjectId @9 :UInt64;
  conditionPercent @10 :UInt8;
  destinationSystemId @11 :UInt64;
  sourceBodyId @12 :UInt32;
  sourceLodeId @13 :UInt64;
  acquisitionKind @14 :CargoAcquisitionKind;
  acquisitionMarketId @15 :UInt64;
  exportTariffPaid @16 :Bool;
  valuationBasisPerTon @17 :UInt64;
}

enum CargoAcquisitionKind {
  purchased @0;
  extracted @1;
  captured @2;
  entrusted @3;
  unique @4;
}

struct CargoSaleQuote {
  cargoLotId @0 :UInt64;
  pricePerTon @1 :UInt64;
  priceDistribution @2 :PriceDistribution;
}

struct MarketSnapshot {
  marketRevision @0 :UInt64;
  systemId @1 :UInt64;
  worldName @2 :Text;
  generatedDay @3 :UInt64;
  credits @4 :UInt64;
  cargoUsedMillitons @5 :UInt64;
  cargoCapacityMillitons @6 :UInt64;
  offers @7 :List(MarketOffer);
  cargo @8 :List(CargoLot);
  tradeCodes @9 :List(Text);
  tariffBasisPoints @10 :UInt16;
  localTaskOffers @11 :List(TaskOffer);
  workAssignments @12 :List(WorkAssignment);
  leads @13 :List(MarketLead);
  events @14 :List(MarketEvent);
  cargoSaleQuotes @15 :List(CargoSaleQuote);
  importTariffBasisPoints @16 :UInt16;
  exportTariffBasisPoints @17 :UInt16;
}

enum MarketLeadSide { supplier @0; buyer @1; }
enum MarketLeadState {
  available @0;
  reserved @1;
  performed @2;
  expired @3;
  cancelled @4;
  negotiating @5;
  quoted @6;
  rejected @7;
}
struct MarketLead {
  leadId @0 :UInt64;
  revision @1 :UInt64;
  side @2 :MarketLeadSide;
  state @3 :MarketLeadState;
  systemId @4 :UInt64;
  commodityId @5 :UInt16;
  commodityName @6 :Text;
  quantityMillitons @7 :UInt64;
  pricePerTon @8 :UInt64;
  discoveredSecond @9 :UInt64;
  expiresSecond @10 :UInt64;
  reservationExpiresSecond @11 :UInt64;
  escrowCredits @12 :UInt64;
  source @13 :Text;
  confidencePercent @14 :UInt8;
  counterpartyId @15 :UInt64;
  cargoLotId @16 :UInt64;
  penaltyUntilSecond @17 :UInt64;
  illegal @18 :Bool;
  loaderFeeCredits @19 :UInt64;
}

enum MarketEventKind { shortage @0; surplus @1; disruption @2; recovery @3; }
struct MarketEvent {
  eventId @0 :UInt64;
  kind @1 :MarketEventKind;
  commodityId @2 :UInt16;
  commodityName @3 :Text;
  startSecond @4 :UInt64;
  expiresSecond @5 :UInt64;
  stockMultiplierBasisPoints @6 :UInt16;
  purchaseTierDelta @7 :Int8;
  saleTierDelta @8 :Int8;
  supplierOfferMultiplierBasisPoints @9 :UInt16;
  buyerOfferMultiplierBasisPoints @10 :UInt16;
  carriageOfferMultiplierBasisPoints @11 :UInt16;
  headline @12 :Text;
}

struct ReserveMarketLeadRequest { leadId @0 :UInt64; expectedRevision @1 :UInt64; quantityMillitons @2 :UInt64; }
struct ReleaseMarketReservationRequest { leadId @0 :UInt64; expectedRevision @1 :UInt64; }

struct BuyCargoRequest {
  marketRevision @0 :UInt64;
  offerId @1 :UInt64;
  quantityMillitons @2 :UInt64;
}

struct SellCargoRequest {
  marketRevision @0 :UInt64;
  cargoLotId @1 :UInt64;
  quantityMillitons @2 :UInt64;
  buyerLeadId @3 :UInt64;
}

enum MarketSearchKind {
  supplier @0;
  buyer @1;
  freight @2;
  passengers @3;
}

enum MarketSearchMethod {
  physical @0;
  online @1;
  blackMarket @2;
  hiredBroker @3;
}

enum WorkState {
  scheduled @0;
  completed @1;
  cancelled @2;
  failed @3;
}

struct BeginMarketSearchRequest {
  kind @0 :MarketSearchKind;
  method @1 :MarketSearchMethod;
  personId @2 :UInt64;
  commodityId @3 :UInt16;
  destinationSystemId @4 :UInt64;
  maximumQuantityMillitons @5 :UInt64;
  cargoLotId @6 :UInt64;
}

struct BeginMarketNegotiationRequest {
  leadId @0 :UInt64;
  expectedRevision @1 :UInt64;
  personId @2 :UInt64;
}

struct MarketQuoteActionRequest {
  leadId @0 :UInt64;
  expectedRevision @1 :UInt64;
}

struct CancelWorkAssignmentRequest {
  assignmentId @0 :UInt64;
}

struct WorkAssignment {
  assignmentId @0 :UInt64;
  kind @1 :MarketSearchKind;
  method @2 :MarketSearchMethod;
  personId @3 :UInt64;
  commodityId @4 :UInt16;
  destinationSystemId @5 :UInt64;
  startedSecond @6 :UInt64;
  dueSecond @7 :UInt64;
  state @8 :WorkState;
  resultText @9 :Text;
  leadId @10 :UInt64;
  maximumQuantityMillitons @11 :UInt64;
  cargoLotId @12 :UInt64;
}

enum TaskKind {
  freight @0;
  passenger @1;
  purchaseOrder @2;
  forwardSale @3;
  supplyCommitment @4;
  charter @5;
  courier @6;
  discoveryBounty @7;
  combatBounty @8;
}

enum TaskState {
  claimPending @0;
  accepted @1;
  sourcing @2;
  loading @3;
  inTransit @4;
  awaitingSettlement @5;
  completed @6;
  expired @7;
  cancelled @8;
  defaulted @9;
  disputed @10;
  lossDocumented @11;
}

struct TaskOffer {
  offerId @0 :UInt64;
  revision @1 :UInt64;
  kind @2 :TaskKind;
  title @3 :Text;
  originSystemId @4 :UInt64;
  destinationSystemId @5 :UInt64;
  commodityId @6 :UInt16;
  quantityMillitons @7 :UInt64;
  passengerCount @8 :UInt16;
  paymentCredits @9 :UInt64;
  collateralCredits @10 :UInt64;
  expiresSecond @11 :UInt64;
  deliveryDeadlineSecond @12 :UInt64;
  legal @13 :Bool;
  partialDeliveryAllowed @14 :Bool;
  failurePenaltyCredits @15 :UInt64;
  recurrenceSeconds @16 :UInt64;
  performanceCount @17 :UInt16;
  passengerClass @18 :PassengerClass;
  lateDeductionPerDayCredits @19 :UInt64;
  nonDeliveryLiabilityCredits @20 :UInt64;
  passengerGraceSeconds @21 :UInt64;
  declaredValueCredits @22 :UInt64;
  unavailableReasons @23 :List(Text);
}

enum PassengerClass { none @0; high @1; middle @2; steerage @3; low @4; charter @5; courier @6; }
enum TaskActionKind { cancel @0; returnCustody @1; defaultTask @2; fileDispute @3; withdrawClaim @4; fileLossClaim @5; }
struct TaskActionRequest { taskId @0 :UInt64; expectedRevision @1 :UInt64; action @2 :TaskActionKind; explanation @3 :Text; }

struct TaskRecord {
  taskId @0 :UInt64;
  offer @1 :TaskOffer;
  state @2 :TaskState;
  acceptedSecond @3 :UInt64;
  deliveredQuantityMillitons @4 :UInt64;
  reservedCargoMillitons @5 :UInt64;
  reservedPassengerCount @6 :UInt16;
  reservedCredits @7 :UInt64;
  statusText @8 :Text;
  performancesCompleted @9 :UInt16;
  revision @10 :UInt64;
  claimMessageId @11 :UInt64;
  resultMessageId @12 :UInt64;
  knownResult @13 :Bool;
  loadedSecond @14 :UInt64;
  settledSecond @15 :UInt64;
  insuranceClaimId @16 :UInt64;
  disputeMessageId @17 :UInt64;
  disputeEffect @18 :Int16;
  adjudicationMessageId @19 :UInt64;
  performingShipId @20 :UInt64;
  piracyEncounterId @21 :UInt64;
  piracyIncidentSecond @22 :UInt64;
  piracyContactId @23 :UInt64;
  piracyThreat @24 :EncounterThreat;
  piracyPosture @25 :EncounterPosture;
  piracyQuantityMillitons @26 :UInt64;
  lossClaimDeadlineSecond @27 :UInt64;
  lossClaimEffect @28 :Int16;
}

struct TaskLedger {
  currentSecond @0 :UInt64;
  availableCredits @1 :UInt64;
  reservedCredits @2 :UInt64;
  reservedCargoMillitons @3 :UInt64;
  reservedPassengerCount @4 :UInt16;
  tasks @5 :List(TaskRecord);
  localOffers @6 :List(TaskOffer);
  carriage @7 :CarriageDeclaration;
  routeAssessments @8 :List(TaskRouteAssessment);
}

struct TaskRouteAssessment {
  offerId @0 :UInt64;
  pickupAvailable @1 :Bool;
  pickupArrivalSecond @2 :UInt64;
  deliveryAvailable @3 :Bool;
  deliveryArrivalSecond @4 :UInt64;
}

struct AcceptTaskOfferRequest {
  offerId @0 :UInt64;
  expectedRevision @1 :UInt64;
}

struct CarriageDeclarationRequest {
  expectedPlanRevision @0 :UInt64;
  destinationSystemId @1 :UInt64;
  freightCapacityMillitons @2 :UInt64;
  highBerths @3 :UInt16;
  middleBerths @4 :UInt16;
  steerageBerths @5 :UInt16;
  lowBerths @6 :UInt16;
  acceptElectronicMail @7 :Bool;
}

struct CarriageDeclaration {
  planRevision @0 :UInt64;
  destinationSystemId @1 :UInt64;
  freightCapacityMillitons @2 :UInt64;
  highBerths @3 :UInt16;
  middleBerths @4 :UInt16;
  steerageBerths @5 :UInt16;
  lowBerths @6 :UInt16;
  acceptElectronicMail @7 :Bool;
}

enum ShipTitleKind {
  ownedWithLien @0;
  sponsorOwned @1;
  institutionOwned @2;
  ownedClear @3;
  prizeCustody @4;
  stolenRegistry @5;
  courtImpound @6;
}

enum ManagedShipOrderKind {
  hold @0;
  followActive @1;
  travel @2;
  dock @3;
  sell @4;
}

struct ManagedShipSummary {
  shipId @0 :UInt64;
  name @1 :Text;
  className @2 :Text;
  catalogId @3 :UInt32;
  systemId @4 :UInt64;
  systemName @5 :Text;
  location @6 :Text;
  title @7 :ShipTitleKind;
  active @8 :Bool;
  commandingPersonId @9 :UInt64;
  commandingPersonName @10 :Text;
  standingOrder @11 :ManagedShipOrderKind;
  canAssumeCommand @12 :Bool;
  fuelMillitons @13 :UInt64;
  fuelCapacityMillitons @14 :UInt64;
  cargoUsedMillitons @15 :UInt64;
  cargoCapacityMillitons @16 :UInt64;
  provisionPersonDays @17 :UInt64;
  provisionCapacityPersonDays @18 :UInt64;
  cargo @19 :List(CargoLot);
  ammunition @20 :List(ShipAmmunitionStatus);
  onlineControlled @21 :Bool;
}

struct FleetSnapshot {
  revision @0 :UInt64;
  activeShipId @1 :UInt64;
  ships @2 :List(ManagedShipSummary);
}

struct SetActiveShipRequest {
  expectedRevision @0 :UInt64;
  shipId @1 :UInt64;
}

struct AssignShipCaptainRequest {
  expectedRevision @0 :UInt64;
  shipId @1 :UInt64;
  personId @2 :UInt64;
}

enum StoreTransferKind {
  cargo @0;
  fuel @1;
  ammunition @2;
  provisions @3;
}

struct TransferShipStoresRequest {
  expectedRevision @0 :UInt64;
  fromShipId @1 :UInt64;
  toShipId @2 :UInt64;
  kind @3 :StoreTransferKind;
  cargoLotId @4 :UInt64;
  itemId @5 :Text;
  quantity @6 :UInt64;
}

struct FinanceSnapshot {
  title @0 :ShipTitleKind;
  liquidCredits @1 :UInt64;
  restrictedCredits @2 :UInt64;
  reservedCredits @3 :UInt64;
  originalHullPriceCredits @4 :UInt64;
  principalCredits @5 :UInt64;
  monthlyPaymentCredits @6 :UInt64;
  monthlyInsuranceEscrowCredits @7 :UInt64;
  nextPaymentDueSecond @8 :UInt64;
  graceExpiresSecond @9 :UInt64;
  paidThroughSecond @10 :UInt64;
  inDefault @11 :Bool;
  impoundOrderKnownLocally @12 :Bool;
  creditStatus @13 :Text;
  destinationAssistanceActive @14 :Bool;
  destinationAssistanceExpiresSecond @15 :UInt64;
  currentSecond @16 :UInt64;
  pendingIncome @17 :List(PendingIncome);
}

enum PendingIncomeStage {
  filingToOffice @0;
  remittanceToCaptain @1;
}

enum IncomeEstimateKind {
  projected @0;
  scheduled @1;
  unavailable @2;
}

struct PendingIncome {
  taskId @0 :UInt64;
  paymentCredits @1 :UInt64;
  reservedReleaseCredits @2 :UInt64;
  stage @3 :PendingIncomeStage;
  estimatedResolutionSecond @4 :UInt64;
  estimateKind @5 :IncomeEstimateKind;
}

enum AccountTransactionClass {
  all @0;
  opening @1;
  income @2;
  expense @3;
  transfer @4;
  hold @5;
  financing @6;
}

enum AccountKind {
  liquid @0;
  restrictedOperating @1;
  reserved @2;
  securedPrincipal @3;
}

enum AccountChangeKind {
  increase @0;
  decrease @1;
  balanceForward @2;
}

struct AccountPosting {
  account @0 :AccountKind;
  change @1 :AccountChangeKind;
  amountCredits @2 :UInt64;
  balanceAfterCredits @3 :UInt64;
  shipId @4 :UInt64;
  shipName @5 :Text;
}

struct AccountLedgerEntry {
  entryId @0 :UInt64;
  occurredSecond @1 :UInt64;
  class @2 :AccountTransactionClass;
  summary @3 :Text;
  subjectShipId @4 :UInt64;
  subjectShipName @5 :Text;
  postings @6 :List(AccountPosting);
}

struct AccountLedgerVessel {
  shipId @0 :UInt64;
  shipName @1 :Text;
}

struct AccountLedgerRequest {
  beforeEntryId @0 :UInt64;
  limit @1 :UInt16;
  class @2 :AccountTransactionClass;
  shipId @3 :UInt64;
}

struct AccountLedgerPage {
  currentSecond @0 :UInt64;
  entries @1 :List(AccountLedgerEntry);
  nextBeforeEntryId @2 :UInt64;
  hasMore @3 :Bool;
  vessels @4 :List(AccountLedgerVessel);
}

struct MarketObservation {
  systemId @0 :UInt64;
  systemName @1 :Text;
  commodityId @2 :UInt16;
  commodityName @3 :Text;
  observedSecond @4 :UInt64;
  acquiredSecond @5 :UInt64;
  source @6 :Text;
  confidencePercent @7 :UInt8;
  minimumPricePerTon @8 :UInt64;
  maximumPricePerTon @9 :UInt64;
  minimumAvailableMillitons @10 :UInt64;
  maximumAvailableMillitons @11 :UInt64;
}

struct MarketKnowledge {
  currentSecond @0 :UInt64;
  observations @1 :List(MarketObservation);
}

struct ShipMarketOffer {
  offerId @0 :UInt64;
  catalogId @1 :UInt32;
  className @2 :Text;
  priceCredits @3 :UInt64;
  originalPriceCredits @4 :UInt64;
  used @5 :Bool;
  ageMonths @6 :UInt32;
  visibleConditionPercent @7 :UInt8;
  cargoCapacityMillitons @8 :UInt64;
  jumpRating @9 :UInt8;
  minimumCrew @10 :UInt16;
}

struct ShipMarket {
  generatedDay @0 :UInt64;
  currentShipTradeInCredits @1 :UInt64;
  outstandingLienCredits @2 :UInt64;
  offers @3 :List(ShipMarketOffer);
  commissionableDesigns @4 :List(ShipCommissionDesign);
}

struct ShipCommissionDesign {
  catalogId @0 :UInt32;
  className @1 :Text;
  techLevel @2 :UInt8;
  priceCredits @3 :UInt64;
  depositCredits @4 :UInt64;
  constructionSeconds @5 :UInt64;
  displacementMillitons @6 :UInt64;
  jumpRating @7 :UInt8;
  fuelCapacityMillitons @8 :UInt64;
  jumpFuelMillitons @9 :UInt64;
  cargoCapacityMillitons @10 :UInt64;
  minimumCrew @11 :UInt16;
}

struct PurchaseShipRequest {
  offerId @0 :UInt64;
  tradeInCurrentShip @1 :Bool;
}

struct CommissionShipRequest {
  catalogId @0 :UInt32;
}

struct CrewCandidate {
  candidateId @0 :UInt64;
  role @1 :Text;
  name @2 :Text;
  primarySkill @3 :SkillId;
  skillLevel @4 :Int8;
  monthlySalaryCredits @5 :UInt64;
}

struct CrewMarket {
  generatedDay @0 :UInt64;
  candidates @1 :List(CrewCandidate);
}

struct HireCrewRequest {
  candidateId @0 :UInt64;
}

struct BeginVoyageRequest {
  destinationSystemId @0 :UInt64;
}

struct TravelStatus {
  shipId @0 :UInt64;
  shipName @1 :Text;
  currentSystemId @2 :UInt64;
  currentSystemName @3 :Text;
  destinationSystemId @4 :UInt64;
  destinationSystemName @5 :Text;
  stage @6 :TravelStage;
  currentGameSecond @7 :UInt64;
  dueSecond @8 :UInt64;
  currentFuelMillitons @9 :UInt64;
  jumpFuelMillitons @10 :UInt64;
  clockRateGameSeconds @11 :UInt64;
  clockRateRealSeconds @12 :UInt64;
  planId @13 :UInt64;
  planRevision @14 :UInt64;
  legIndex @15 :UInt16;
  origin @16 :FlightLocus;
  destination @17 :FlightLocus;
  fuelCapacityMillitons @18 :UInt64;
}

struct FlightLocus {
  systemId @0 :UInt64;
  union {
    port @1 :PortLocus;
    jumpLocus @2 :Void;
    bodyId @3 :UInt32;
    deepSpace @4 :Vector3Parsecs;
  }
  jumpRole @5 :JumpLocusRole;
  remoteArrival @6 :Bool;
}

enum JumpLocusRole {
  departure @0;
  arrival @1;
}

struct Vector3Parsecs {
  # Earth-centred Galactic axes. Positive is coreward, spinward, and north.
  coreward @0 :Float64;
  spinward @1 :Float64;
  north @2 :Float64;
}

struct PortLocus {
  worldId @0 :UInt64;
  facilityId @1 :UInt64;
}

enum TravelStage {
  docked @0;
  departingForJump @1;
  jumpSpace @2;
  approachingStarport @3;
  refit @4;
  properRepair @5;
  gasGiantSkim @6;
  wildernessWater @7;
  holding @8;
  encounter @9;
  beltProspecting @10;
  beltSurvey @11;
  beltMining @12;
  beltRefining @13;
  beltRecovery @14;
  beltEgress @15;
  fuelProcessing @16;
  maneuvering @17;
}

enum WaypointAuthority {
  hold @0;
  terminal @1;
  through @2;
}

enum FuelOperation {
  gasGiant @0;
  wildernessWater @1;
  buyRefined @2;
  buyUnrefined @3;
}

struct FlightPlanAction {
  union {
    hold @0 :Void;
    jump @1 :JumpPlanAction;
    dock @2 :PortLocus;
    fuel @3 :FuelPlanAction;
    jumpCoordinates @4 :CoordinateJumpPlanAction;
    beltCycle @5 :UInt32;
    refineFuel @6 :UInt64;
  }
}

enum JumpNavigationMethod {
  onboard @0;
  commercialTape @1;
}

struct JumpPlanAction {
  destinationSystemId @0 :UInt64;
  navigation @1 :JumpNavigationMethod;
  proceedOnKnownBadPlot @2 :Bool;
  remoteArrival @3 :Bool;
  departureLocusArrival @4 :Bool;
}

struct CoordinateJumpPlanAction {
  destination @0 :Vector3Parsecs;
  navigation @1 :JumpNavigationMethod;
  proceedOnKnownBadPlot @2 :Bool;
}

struct FuelPlanAction {
  operation @0 :FuelOperation;
  quantityMillitons @1 :UInt64;
  refineCollected @2 :Bool = true;
}

struct FuelOperationTiming {
  stepIndex @0 :UInt16;
  roundTripSeconds @1 :UInt64;
  collectionSeconds @2 :UInt64;
  processingSeconds @3 :UInt64;
  failedProcessingSeconds @4 :UInt64;
  normalTotalSeconds @5 :UInt64;
  failedTotalSeconds @6 :UInt64;
  outputRefined @7 :Bool;
}

struct FlightPlanStep {
  locus @0 :FlightLocus;
  authority @1 :WaypointAuthority;
  action @2 :FlightPlanAction;
  terminal @3 :Bool;
}

enum EncounterPosture {
  fight @0;
  flee @1;
  comply @2;
  surrender @3;
  board @4;
  pursue @5;
  continueCourse @6;
}

enum EncounterFallback {
  surrender @0;
  abandon @1;
  jettisonCargo @2;
  breakOff @3;
}

struct EncounterPolicy {
  hostilePosture @0 :EncounterPosture;
  hostileFallbacks @1 :List(EncounterFallback);
  complyWithInspection @2 :Bool;
  reportDistress @3 :Bool;
  assistDistress @4 :Bool;
  standingOrders @5 :List(EncounterStandingOrder);
}

enum EncounterFightMode {
  never @0;
  always @1;
  estimatedAtLeast @2;
}

struct EncounterStandingOrder {
  kind @0 :EncounterKind;
  ordinaryPosture @1 :EncounterPosture;
  fightMode @2 :EncounterFightMode;
  minimumOutlookPercent @3 :UInt8;
}

struct EncounterPolicyDefaultSnapshot {
  shipId @0 :UInt64;
  revision @1 :UInt64;
  policy @2 :EncounterPolicy;
}

struct SetEncounterPolicyDefaultRequest {
  expectedRevision @0 :UInt64;
  policy @1 :EncounterPolicy;
  acknowledgeNonhostileFight @2 :Bool;
}

struct FlightPlanProposal {
  expectedPlanRevision @0 :UInt64;
  steps @1 :List(FlightPlanStep);
  policy @2 :EncounterPolicy;
  preserveActiveStep @3 :Bool;
}

struct CommitFlightPlanRequest {
  proposal @0 :FlightPlanProposal;
  previewHash @1 :Data;
  acknowledgeWarnings @2 :Bool;
}

struct FlightPlanWarning {
  code @0 :Text;
  message @1 :Text;
  stepIndices @2 :List(UInt16);
}

struct FlightPlanPreview {
  proposal @0 :FlightPlanProposal;
  previewHash @1 :Data;
  elapsedSeconds @2 :UInt64;
  fuelMillitons @3 :UInt64;
  warnings @4 :List(FlightPlanWarning);
  carriageOffers @5 :List(TaskOffer);
  carriageRevenueCredits @6 :UInt64;
  carriageBrokerFeesCredits @7 :UInt64;
  fuelTimings @8 :List(FuelOperationTiming);
}

enum FlightPlanState {
  inactive @0;
  active @1;
  held @2;
  checkpoint @3;
  encounter @4;
  completed @5;
  terminal @6;
}

struct FlightPlanSnapshot {
  planId @0 :UInt64;
  revision @1 :UInt64;
  currentStep @2 :UInt16;
  state @3 :FlightPlanState;
  steps @4 :List(FlightPlanStep);
  policy @5 :EncounterPolicy;
  suspensionReason @6 :Text;
}

enum CheckpointKind {
  portDeparture @0;
  inhabitedWorld @1;
  gasGiant @2;
  jumpArrival @3;
  jumpDeparture @4;
  deepSpace @5;
}

struct CheckpointSnapshot {
  checkpointId @0 :UInt64;
  planId @1 :UInt64;
  planRevision @2 :UInt64;
  stepIndex @3 :UInt16;
  locus @4 :FlightLocus;
  kind @5 :CheckpointKind;
  readySecond @6 :UInt64;
  acknowledged @7 :Bool;
}

struct AcknowledgeCheckpointRequest {
  checkpointId @0 :UInt64;
}

enum EncounterKind {
  routineTraffic @0;
  trafficControl @1;
  inspection @2;
  distress @3;
  derelict @4;
  hazard @5;
  hostile @6;
  military @7;
  departingContact @8;
}

enum EncounterState {
  awaitingPosture @0;
  resolving @1;
  resolved @2;
}

struct EncounterContact {
  contactId @0 :UInt64;
  shipName @1 :Text;
  className @2 :Text;
  transponder @3 :Text;
  role @4 :Text;
  range @5 :Text;
  confidencePercent @6 :UInt8;
  resolution @7 :EncounterResolution;
  declaredClassName @8 :Text;
}

enum EncounterResolution { radioOnly @0; transponderOnly @1; approximate @2; identified @3; }
enum EncounterAuthority { none @0; pirate @1; trafficControl @2; customs @3; naval @4; warrant @5; }
enum EncounterThreat { unknown @0; favorable @1; comparable @2; dangerous @3; overwhelming @4; }

struct EncounterDemand {
  present @0 :Bool;
  playerOwnedPercent @1 :UInt8;
  playerOwnedMillitons @2 :UInt64;
  entrustedMillitons @3 :UInt64;
  uniqueObjectCount @4 :UInt16;
  text @5 :Text;
  entrustedLiabilityCredits @6 :UInt64;
}

struct EncounterSnapshot {
  encounterId @0 :UInt64;
  revision @1 :UInt64;
  kind @2 :EncounterKind;
  state @3 :EncounterState;
  startedSecond @4 :UInt64;
  nextTurnSecond @5 :UInt64;
  turn @6 :UInt16;
  contact @7 :EncounterContact;
  summary @8 :Text;
  authority @9 :EncounterAuthority;
  threat @10 :EncounterThreat;
  demand @11 :EncounterDemand;
  availablePostures @12 :List(EncounterPosture);
  availableFallbacks @13 :List(EncounterFallback);
  responseDeadlineSecond @14 :UInt64;
  estimatedCombatOutlookPercent @15 :UInt8;
}

struct ResolveEncounterRequest {
  encounterId @0 :UInt64;
  expectedRevision @1 :UInt64;
  posture @2 :EncounterPosture;
  fallbacks @3 :List(EncounterFallback);
}

struct EncounterResult {
  encounterId @0 :UInt64;
  resolved @1 :Bool;
  terminal @2 :Bool;
  outcome @3 :Text;
  turns @4 :UInt16;
  cargoLostMillitons @5 :UInt64;
  fuelLostMillitons @6 :UInt64;
  damageHits @7 :UInt16;
}

enum CommandLossKind {
  destroyed @0;
  captured @1;
  surrendered @2;
  abandoned @3;
  bankruptcy @4;
}

enum CaptainFate { survived @0; dead @1; }

struct TerminalReport {
  encounterId @0 :UInt64;
  revision @1 :UInt64;
  acknowledged @2 :Bool;
  startedSecond @3 :UInt64;
  resolvedSecond @4 :UInt64;
  systemId @5 :UInt64;
  systemName @6 :Text;
  location @7 :Text;
  contact @8 :EncounterContact;
  authority @9 :EncounterAuthority;
  threat @10 :EncounterThreat;
  standingOrdersUsed @11 :Bool;
  hasPosture @12 :Bool;
  posture @13 :EncounterPosture;
  fallbacks @14 :List(EncounterFallback);
  automatedCombatUsed @15 :Bool;
  outcome @16 :Text;
  shipName @17 :Text;
  lossKind @18 :CommandLossKind;
  ownedCargoLostMillitons @19 :UInt64;
  entrustedCargoLostMillitons @20 :UInt64;
  uniqueObjectsLost @21 :UInt16;
  fuelLostMillitons @22 :UInt64;
  passengersAffected @23 :UInt16;
  damageHits @24 :UInt16;
  captainName @25 :Text;
  captainFate @26 :CaptainFate;
  otherCrewTotal @27 :UInt16;
  otherCrewDead @28 :UInt16;
  otherCrewInjured @29 :UInt16;
  otherCrewSurviving @30 :UInt16;
  recoveryReadySecond @31 :UInt64;
  successorRequired @32 :Bool;
  incidentLog @33 :List(Text);
}

struct AcknowledgeTerminalReportRequest {
  encounterId @0 :UInt64;
  expectedRevision @1 :UInt64;
}

enum OperationalDamageCause {
  jumpTransition @0;
  fuelProcessing @1;
  maintenanceNeglect @2;
}

struct OperationalDamageReport {
  present @0 :Bool;
  reportId @1 :UInt64;
  occurredSecond @2 :UInt64;
  shipId @3 :UInt64;
  shipName @4 :Text;
  cause @5 :OperationalDamageCause;
  originSystemId @6 :UInt64;
  originSystemName @7 :Text;
  destinationSystemId @8 :UInt64;
  destinationSystemName @9 :Text;
  inaccurateExtraDays @10 :UInt8;
  misjump @11 :Bool;
  subsystemId @12 :UInt16;
  subsystemKind @13 :ShipSubsystemKind;
  subsystemLabel @14 :Text;
  damageHits @15 :UInt16;
  sustainedHits @16 :UInt16;
  maximumHits @17 :UInt16;
  operationalEffect @18 :Text;
}

struct AcknowledgeOperationalDamageReportRequest {
  reportId @0 :UInt64;
}

enum CombatRange {
  adjacent @0;
  close @1;
  short @2;
  medium @3;
  long @4;
  veryLong @5;
  distant @6;
}

enum CombatDisposition {
  active @0;
  withdrawing @1;
  surrenderOffered @2;
  surrendered @3;
  abandoned @4;
  captured @5;
  destroyed @6;
}

enum CombatObjective {
  survive @0;
  withdraw @1;
  defeat @2;
  capture @3;
  protect @4;
  inspect @5;
}

enum CombatActionKind {
  hold @0;
  coordinate @1;
  increaseInitiative @2;
  evasiveManeuvers @3;
  lineUpShot @4;
  rangeCheckClose @5;
  rangeCheckOpen @6;
  breakPursuit @7;
  sensorTargeting @8;
  electronicWarfare @9;
  damageControl @10;
  attack @11;
  board @12;
  prepareJump @13;
  launchEscapeCraft @14;
  offerSurrender @15;
  acceptSurrender @16;
  inspectContact @17;
  pursuit @18;
}

enum CombatReaction {
  dodge @0;
  pointDefense @1;
  fireSand @2;
  triggerNuclearDamper @3;
  triggerMesonScreen @4;
}

struct CombatAction {
  kind @0 :CombatActionKind;
  mountId @1 :UInt16;
  targetVesselId @2 :UInt64;
  actorPersonId @3 :UInt64;
}

struct CombatReactionOrder {
  kind @0 :CombatReaction;
  actorPersonId @1 :UInt64;
}

struct CombatOrderSet {
  combatId @0 :UInt64;
  viewRevision @1 :UInt64;
  actions @2 :List(CombatAction);
  reactions @3 :List(CombatReactionOrder);
  useTacticalController @4 :Bool;
  speedAdjustment @5 :Int16;
  speedActorPersonId @6 :UInt64;
}

struct CombatAutomationPolicy {
  expectedRevision @0 :UInt64;
  minimumVictoryPercent @1 :UInt8;
  objective @2 :CombatObjective;
  permitSurrender @3 :Bool;
  permitAbandonShip @4 :Bool;
}

struct CombatWeaponMount {
  mountId @0 :UInt16;
  label @1 :Text;
  weapons @2 :List(Text);
  damageHits @3 :UInt8;
  ammunitionRemaining @4 :UInt32;
}

struct CombatParticipant {
  vesselId @0 :UInt64;
  side @1 :UInt16;
  name @2 :Text;
  className @3 :Text;
  initiative @4 :Int16;
  thrust @5 :UInt8;
  hullRemaining @6 :UInt16;
  structureRemaining @7 :UInt16;
  armorRemaining @8 :UInt16;
  disposition @9 :CombatDisposition;
  weapons @10 :List(CombatWeaponMount);
  commanded @11 :Bool;
  playerOwned @12 :Bool;
  onlineControlled @13 :Bool;
  speed @14 :Int16;
  pursuitTargetVesselId @15 :UInt64;
  pursuitAttackBonus @16 :UInt8;
}

struct CombatActor {
  personId @0 :UInt64;
  name @1 :Text;
  station @2 :Text;
  available @3 :Bool;
  actionBudget @4 :UInt8;
  allowedActions @5 :List(CombatActionKind);
  allowedReactions @6 :List(CombatReaction);
}

struct CombatSnapshot {
  combatId @0 :UInt64;
  revision @1 :UInt64;
  round @2 :UInt16;
  roundStartedSecond @3 :UInt64;
  orderDueSecond @4 :UInt64;
  orderWindowRealMilliseconds @5 :UInt64;
  range @6 :CombatRange;
  participants @7 :List(CombatParticipant);
  defaultOrder @8 :CombatOrderSet;
  policy @9 :CombatAutomationPolicy;
  playerOrderSubmitted @10 :Bool;
  complete @11 :Bool;
  log @12 :List(Text);
  actors @13 :List(CombatActor);
}

enum CombatCareerMode { independent @0; navy @1; privateer @2; pirate @3; }
enum CareerOpportunityKind { navalOrder @0; privateerCommission @1; pirateLead @2; pirateCommission @3; }
enum CareerOpportunityState { offered @0; accepted @1; succeeded @2; failed @3; expired @4; reporting @5; }
enum CareerObjectiveKind { patrol @0; inspect @1; escort @2; intercept @3; capture @4; seizeCargo @5; }
enum CareerObjectiveEvidenceKind {
  none @0; patrolLog @1; inspectionRecord @2; escortRelease @3;
  targetDrivenOff @4; targetCaptured @5; cargoSecured @6; cargoDelivered @7;
}
enum PrizeStatus { secured @0; claimInTransit @1; awaitingAdjudication @2; adjudicated @3; readyToFence @4; settled @5; seized @6; }
enum WarrantStatus { filed @0; propagating @1; active @2; revoked @3; satisfied @4; }

struct CareerOpportunity {
  opportunityId @0 :UInt64; kind @1 :CareerOpportunityKind; state @2 :CareerOpportunityState;
  issuedSystemId @3 :UInt64; targetSystemId @4 :UInt64; targetContactId @5 :UInt64;
  issuedSecond @6 :UInt64; expiresSecond @7 :UInt64; rewardCredits @8 :UInt64;
  servicePoints @9 :UInt16; authority @10 :Text; objective @11 :Text;
  objectiveKind @12 :CareerObjectiveKind; evidenceKind @13 :CareerObjectiveEvidenceKind;
  evidenceSecond @14 :UInt64; orderMessageId @15 :UInt64;
  evidenceVesselId @16 :UInt64; reportMessageId @17 :UInt64;
}
struct PrizeRecord {
  prizeId @0 :UInt64; capturedVesselId @1 :UInt64; catalogId @2 :UInt32; name @3 :Text;
  grossValueCredits @4 :UInt64; realizableValueCredits @5 :UInt64; conditionPercent @6 :UInt8;
  status @7 :PrizeStatus; securedSecond @8 :UInt64; claimMessageId @9 :UInt64;
  settlementCredits @10 :UInt64; advanceCredits @11 :UInt64;
  survivingCrewCount @12 :UInt16;
}
struct WarrantRecord {
  warrantId @0 :UInt64; issuingPolityId @1 :UInt64; originSystemId @2 :UInt64;
  filedSecond @3 :UInt64; messageId @4 :UInt64; severity @5 :UInt8; bountyCredits @6 :UInt64;
  evidencePercent @7 :UInt8; status @8 :WarrantStatus; accusation @9 :Text;
  resolutionMessageId @10 :UInt64; resolvedSecond @11 :UInt64; resolvingSystemId @12 :UInt64;
}
enum WarrantAssociationKind { historical @0; reportedAboard @1; confirmedAboard @2; wantedVessel @3; }
enum BountyCustodyState { atLarge @0; heldAboard @1; settled @2; }
struct KnownWarrant {
  warrantId @0 :UInt64;
  subjectPersonId @1 :UInt64;
  subjectName @2 :Text;
  subjectRole @3 :Text;
  accusation @4 :Text;
  bountyCredits @5 :UInt64;
  severity @6 :UInt8;
  evidencePercent @7 :UInt8;
  issuingPolityId @8 :UInt64;
  originSystemId @9 :UInt64;
  filedSecond @10 :UInt64;
  associatedShipId @11 :UInt64;
  associatedShipName @12 :Text;
  associatedTransponder @13 :Text;
  lastKnownSystemId @14 :UInt64;
  association @15 :WarrantAssociationKind;
  custody @16 :BountyCustodyState;
  generatedTarget @17 :Bool;
}
struct PirateCruise {
  revision @0 :UInt64; active @1 :Bool; huntingSystemId @2 :UInt64; endsSecond @3 :UInt64;
  crewSharePercent @4 :UInt8; shipFundPercent @5 :UInt8; prohibitedTargets @6 :Text;
}
struct CombatCareerSnapshot {
  revision @0 :UInt64; mode @1 :CombatCareerMode; rank @2 :Text; servicePoints @3 :UInt16;
  monthlySalaryCredits @4 :UInt64; nextNavalBoardSecond @5 :UInt64; publicHeat @6 :UInt32;
  underworldStanding @7 :Int16; opportunities @8 :List(CareerOpportunity); prizes @9 :List(PrizeRecord);
  warrants @10 :List(WarrantRecord); cruise @11 :PirateCruise; localEnforcementSummary @12 :Text;
  localContacts @13 :List(TrafficContact);
  crewPressure @14 :UInt16;
  systemContacts @15 :List(TrafficContact);
  interceptionWatch @16 :InterceptionWatchStatus;
  hasInterceptionWatch @17 :Bool;
  knownWarrants @18 :List(KnownWarrant);
}
struct CareerOpportunityRequest { opportunityId @0 :UInt64; expectedRevision @1 :UInt64; }
enum InterceptionPurpose {
  armedAttack @0;
  boardingInspection @1;
  arrest @2;
}
struct EngageTrafficContactRequest {
  contactId @0 :UInt64;
  expectedCareerRevision @1 :UInt64;
  purpose @2 :InterceptionPurpose;
}
enum InterceptionWatchFilterKind { namedVessel @0; craftClass @1; allCraft @2; }
struct InterceptionWatchStatus {
  startedSecond @0 :UInt64;
  targetContactId @1 :UInt64;
  targetCatalogId @2 :UInt32;
  targetShipName @3 :Text;
  filter @4 :InterceptionWatchFilterKind;
  locus @5 :FlightLocus;
  purpose @6 :InterceptionPurpose;
}
struct InterceptionWatchRequest {
  expectedCareerRevision @0 :UInt64;
  union {
    cancel @1 :Void;
    allCraft @2 :Void;
    catalogId @3 :UInt32;
  }
  purpose @4 :InterceptionPurpose;
}
struct PirateCruiseRequest { expectedRevision @0 :UInt64; active @1 :Bool; huntingSystemId @2 :UInt64; endsSecond @3 :UInt64; crewSharePercent @4 :UInt8; shipFundPercent @5 :UInt8; prohibitedTargets @6 :Text; }
enum PrizeSettlementMethod {
  fileClaim @0;
  takeAdvance @1;
  fence @2;
  courtSale @3;
  keepPrize @4;
  launderRegistry @5;
}
struct PrizeSettlementRequest { prizeId @0 :UInt64; expectedCareerRevision @1 :UInt64; method @2 :PrizeSettlementMethod; }
struct WarrantSettlementRequest { warrantId @0 :UInt64; expectedCareerRevision @1 :UInt64; }
struct CombatCareerModeRequest { mode @0 :CombatCareerMode; expectedRevision @1 :UInt64; }
struct RecoverCommandRequest { successorName @0 :Text; }
struct AbandonPlayerRequest { confirmation @0 :Text; }

enum MessageClass {
  agencyNews @0;
  publicService @1;
  contractOffer @2;
  trafficNotice @3;
  privateMessage @4;
}

enum MessageImportance {
  routine @0;
  notable @1;
  important @2;
  headline @3;
}

enum MessageClassification {
  unreviewed @0;
  ignored @1;
  reviewLater @2;
  actioned @3;
  archived @4;
}

struct MessageItem {
  messageId @0 :UInt64;
  originSystemId @1 :UInt64;
  originSystemName @2 :Text;
  createdSecond @3 :UInt64;
  availableSecond @4 :UInt64;
  expiresSecond @5 :UInt64;
  class @6 :MessageClass;
  subject @7 :Text;
  classification @8 :MessageClassification;
  previouslySeen @9 :Bool;
  expired @10 :Bool;
  body @11 :Text;
  offerId @12 :UInt64;
  offerAvailable @13 :Bool;
  offerRevision @14 :UInt64;
  importance @15 :MessageImportance;
  actionKind @16 :MessageActionKind;
  actionReferenceId @17 :UInt64;
}

enum MessageActionKind { none @0; claimOffer @1; reviewTask @2; reviewOperations @3; reviewFinance @4; reviewMapping @5; }

struct ArrivalPacket {
  systemId @0 :UInt64;
  systemName @1 :Text;
  arrivalSecond @2 :UInt64;
  mailbagId @3 :UInt64;
  mailDelivered @4 :UInt64;
  mailForwarded @5 :UInt64;
  mailExpired @6 :UInt64;
  stipendCredits @7 :UInt64;
  items @8 :List(MessageItem);
  newArrival @9 :Bool;
  mappingStatus @10 :SystemMappingStatus;
}

struct MessageManagement {
  items @0 :List(MessageItem);
  filters @1 :List(MessageFilter);
}

struct MessageFilter {
  class @0 :MessageClass;
  minimumImportance @1 :MessageImportance;
}

struct SetMessageFilterRequest {
  class @0 :MessageClass;
  minimumImportance @1 :MessageImportance;
}

struct SetMessageClassificationRequest {
  messageId @0 :UInt64;
  classification @1 :MessageClassification;
}

enum PrivateRecipientKind { system @0; captain @1; }
struct SendPrivateMessageRequest {
  recipientKind @0 :PrivateRecipientKind;
  destinationSystemId @1 :UInt64;
  recipient @2 :PlayerIdentity;
  encryptionKeyId @3 :UInt64;
  ttlWeeks @4 :UInt16;
  subject @5 :Text;
  body @6 :Text;
}

enum RadioTransmissionKind {
  playerBroadcast @0;
  inspectionOrder @1;
  boardingOrder @2;
  pirateDemand @3;
}

struct RadioInboxEntry {
  receptionId @0 :UInt64;
  transmissionId @1 :UInt64;
  receivingShipId @2 :UInt64;
  senderShipId @3 :UInt64;
  senderShipName @4 :Text;
  senderTransponder @5 :Text;
  sender @6 :PlayerIdentity;
  emittedSecond @7 :UInt64;
  receivedSecond @8 :UInt64;
  expiresSecond @9 :UInt64;
  kind @10 :RadioTransmissionKind;
  actionable @11 :Bool;
  actionReferenceId @12 :UInt64;
}

struct RadioMute {
  sender @0 :PlayerIdentity;
}

struct SystemRadioSnapshot {
  shipId @0 :UInt64;
  systemId @1 :UInt64;
  currentSecond @2 :UInt64;
  canTransmit @3 :Bool;
  unavailableReason @4 :Text;
  entries @5 :List(RadioInboxEntry);
  mutes @6 :List(RadioMute);
}

struct RadioContent {
  receptionId @0 :UInt64;
  transmissionId @1 :UInt64;
  body @2 :Text;
}

struct TransmitSystemRadioRequest { body @0 :Text; }
struct RadioReceptionRequest { receptionId @0 :UInt64; }
struct SetRadioMuteRequest { sender @0 :PlayerIdentity; muted @1 :Bool; }

enum InsuranceKind { destinationAssistance @0; }
struct PurchaseInsuranceRequest { kind @0 :InsuranceKind; enabled @1 :Bool; }

struct MisappropriateRestrictedCreditsRequest { amount @0 :UInt64; }

enum SystemMappingState {
  knownPublic @0;
  unresolved @1;
  publicDispatched @2;
  directDispatched @3;
  withheld @4;
  secret @5;
}

enum SystemMappingChoice {
  publicNotification @0;
  directEarth @1;
  withhold @2;
  withholdSecret @3;
}

struct SystemMappingStatus {
  systemId @0 :UInt64;
  state @1 :SystemMappingState;
  dispatchMessageId @2 :UInt64;
  changedSecond @3 :UInt64;
}

struct SetSystemMappingDisclosureRequest {
  systemId @0 :UInt64;
  choice @1 :SystemMappingChoice;
}

enum ErrorCode {
  invalidCommand @0;
  staleSession @1;
  malformedMessage @2;
  unsupportedVersion @3;
  internalFailure @4;
}

struct Error {
  code @0 :ErrorCode;
  message @1 :Text;
}

struct Event {
  committedSequence @0 :UInt64;
  union {
    sessionReplaced @1 :Void;
    serverStopping @2 :Void;
    phaseChanged @3 :PhaseChanged;
    trafficSnapshot @4 :TrafficSnapshot;
    trafficMovement @5 :TrafficMovement;
    checkpointReady @6 :CheckpointSnapshot;
    encounterReady @7 :EncounterSnapshot;
    radioUnread @8 :RadioUnread;
    operationalDamageReady @9 :OperationalDamageReport;
  }
}

struct RadioUnread {
  shipId @0 :UInt64;
  unreadCount @1 :UInt64;
}

enum TrafficMovementKind {
  arrival @0;
  departure @1;
  present @2;
}

enum TrafficContactResolution {
  transponderOnly @0;
  approximate @1;
  identified @2;
}

enum TrafficAttachment {
  spaceborne @0;
  berthed @1;
  landed @2;
}

struct TrafficContact {
  contactId @0 :UInt64;
  catalogId @1 :UInt32;
  className @2 :Text;
  shipName @3 :Text;
  transponder @4 :Text;
  operatorName @5 :Text;
  role @6 :Text;
  displacementMillitons @7 :UInt64;
  originSystemId @8 :UInt64;
  destinationSystemId @9 :UInt64;
  movement @10 :TrafficMovementKind;
  edgeSecond @11 :UInt64;
  resolution @12 :TrafficContactResolution;
  confidencePercent @13 :UInt8;
  playerOwned @14 :Bool;
  onlineControlled @15 :Bool;
  attachment @16 :TrafficAttachment;
}

struct TrafficSnapshot {
  systemId @0 :UInt64;
  systemName @1 :Text;
  observedSecond @2 :UInt64;
  contacts @3 :List(TrafficContact);
}

struct TrafficMovement {
  systemId @0 :UInt64;
  observedSecond @1 :UInt64;
  contact @2 :TrafficContact;
}

struct PhaseChanged {
  revision @0 :UInt64;
  phase @1 :Phase;
  travelStatus @2 :TravelStatus;
}

struct Cancel {
  requestId @0 :UInt64;
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
