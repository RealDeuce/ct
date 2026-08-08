#include "ct/door_help.hpp"

#include <array>
#include <stdexcept>

namespace ct {
namespace {

constexpr std::array<DoorHelp, static_cast<size_t>(DoorHelpTopic::Count)> HELP{{
   {
      "Cepheus Trader help",
      "Press ? at an advertised prompt for help about the screen or decision in front of you. "
      "Help explains known rules and consequences; it does not reveal facts your captain has "
      "not learned.\n\n"
      "The main areas are captain creation, ships and crew, cargo and contracts, messages and "
      "mail, navigation and fuel, and encounters and combat. The Captain's Command Console is "
      "the in-game help index and remains available while docked or travelling.\n\n"
      "Enter usually accepts, refreshes, or returns to the preceding screen. Q cancels a pending "
      "choice. Leaving the game requires a separate confirmation.",
   },
   {
      "Registering a captain",
      "A BBS account may register one captain. Registration creates the persistent identity used "
      "for ships, money, mail, obligations, discoveries, service, and legal records.\n\n"
      "You will choose characteristics and skills, select a career-backed starting ship package, "
      "and review its crew. Starting packages differ in ownership, debt, operating authority, "
      "cargo capacity, jump capability, armament, and required crew.\n\n"
      "Nothing is committed until the final registration confirmation. Return to earlier screens "
      "to compare packages before accepting one.",
   },
   {
      "Characteristics and skills",
      "STR, DEX, END, INT, EDU, and CHA describe the captain. Every three points change the usual "
      "task modifier by one. The point-buy screen must finish with exactly the displayed budget "
      "spent.\n\n"
      "Skills provide training for particular tasks. A zero rating is trained competence; positive "
      "ratings provide increasing expertise. Jack of All Trades reduces penalties for untrained "
      "work but does not replace a required professional qualification.\n\n"
      "Training targets progress in whole training weeks while the person remains eligible to train.",
   },
   {
      "Starting careers",
      "A career determines the authority and financial structure of the offered command. Trader "
      "packages emphasize commerce, privateer packages combine commerce with licensed force, and "
      "naval packages place a ship under service authority.\n\n"
      "Career is not merely a difficulty label. Read the package description, title and authority, "
      "starting reserve, debt or bond, operating costs, and exit terms before selecting it.\n\n"
      "Changing later may require paying obligations, returning an issued vessel, acquiring title, "
      "or accepting legal consequences.",
   },
   {
      "Starting ship offers",
      "Jump rating limits the distance of one jump, not the length of a multi-jump course. Thrust "
      "affects in-system travel and tactical movement. Cargo capacity is what remains after the "
      "ship's installed systems, small craft, crew spaces, and stores.\n\n"
      "Inspect an offer for its title, authority, debt, fuel, ammunition, crew, passenger space, and "
      "maintenance burden. A more powerful vessel can be much more expensive to operate and repair.\n\n"
      "The course plotter can later include fuel purchases and lawful frontier collection, but a "
      "ship still needs enough tankage and money to execute those stops.",
   },
   {
      "Crew and personnel",
      "A ship needs qualified people for its watches and specialist work. The roster shows skills, "
      "assignment, condition, morale, pay, and training. Unfilled or incapacitated positions can "
      "make actions slower, harder, or unavailable.\n\n"
      "Awake crew and passengers consume provisions. Low-berth passengers do not consume normal "
      "provisions while berthed. Payroll, arrears, medical care, shore leave, transfer, discharge, "
      "and training all persist.\n\n"
      "Discharge and reassignment can leave the ship without a legal or competent watch, so review "
      "coverage before confirming them.",
   },
   {
      "Captain's Command Console",
      "This is the help index and the universal entrance to six managers. Crew covers people and "
      "training; Ship covers condition, stores, and fleet custody; Tasks covers offers and accepted "
      "obligations; Messages covers physical correspondence; Known Universe covers charts and route "
      "planning; Operations covers service, traffic, and combat activity.\n\n"
      "The managers are available while docked and during scheduled travel, although particular "
      "actions depend on location, facilities, authority, and current ship state.\n\n"
      "Press the manager's letter, Enter to return to the operational screen, or Q to request leaving "
      "the game.",
   },
   {
      "Ship management",
      "Ship management reports title, command, location, fuel, cargo, ammunition, provisions, crew "
      "capacity, installed systems, damage, repairs, and other vessels in the fleet.\n\n"
      "Damage and temporary battlefield loss are distinct. Field recovery may restore disabled "
      "capability; proper repair restores lasting condition and needs suitable facilities, parts, "
      "money, and time.\n\n"
      "Transferring stores or command changes real custody. Verify source, destination, capacity, "
      "captain assignment, and title before confirming a fleet action.",
   },
   {
      "Tasks, charters, and obligations",
      "An offer received through mail may already be stale. Claiming it signs a response and reserves "
      "the displayed collateral or capacity while that response physically travels to the issuing "
      "office. A claim is not yet an award.\n\n"
      "The first valid claim accepted by the issuer wins. The winner receives confirmation and may "
      "take custody of the consignment at the origin. Losing claims receive a decline that releases "
      "their reserves. Your captain carries a signed claim personally when travelling, so the captain "
      "cannot arrive before that claim.\n\n"
      "Delivery, deadline, default, abandonment, collateral, and liability terms remain binding once "
      "the award is accepted.",
   },
   {
      "Messages and physical mail",
      "Messages are dated knowledge carried between systems. Delivery is not instantaneous, and a "
      "remote office or market may act before its reply reaches you. Old local copies can therefore "
      "show offers, warrants, prices, or closures that are no longer current at their source.\n\n"
      "A ship can carry ordinary mailbags. Private instruments such as claims, awards, warrants, and "
      "settlements have explicit origins, destinations, custody, and delivery records. The captain "
      "will hand-deliver personally carried instruments on arrival even without an ordinary mailbag.\n\n"
      "Ignoring a message changes its presentation, not the underlying obligation or event.",
   },
   {
      "Known Universe and charts",
      "Charts contain only knowledge available to this captain. System names, coordinates, ports, "
      "fuel sources, market reports, and routes can be incomplete or dated. Mapping and disclosure "
      "move through physical records and mail.\n\n"
      "Use filters and pages to inspect known systems, then open a system for details or route "
      "planning. A missing route can mean insufficient jump rating, unavailable lawful fuel, "
      "insufficient funds, or simply missing chart knowledge.\n\n"
      "The fastest and cheapest plots may choose different fuel sources and stopping points.",
   },
   {
      "Operations and service",
      "Operations covers local contacts, naval or private authority, orders, reports, warrants, and "
      "combat-related activity. Local contacts contain only vessels sharing the ship's present "
      "traffic locus, such as a port, Jump locus, or frontier-fuel body. Ordinary interplanetary "
      "flight and Jump space normally have no local contacts.\n\n"
      "Civilian identity and registry details come from transponders. Hull classification and "
      "tonnage are sensor observations whose confidence depends on the fitted electronics and "
      "sensor damage. System-wide traffic-control reports are a separate transponder picture; they "
      "do not imply that a reported ship is locally detectable or interceptable.\n\n"
      "Interception, piracy, mutiny, and misuse of an issued command can create durable legal and "
      "career consequences. Read the displayed authority and confirmation text before proceeding.\n\n"
      "Service orders and reports are instruments that may need physical delivery before another "
      "office recognizes them.",
   },
   {
      "Docked operations",
      "The docked menu lists services actually present at this port. Cargo, fuel, repairs, crew, "
      "banking, and authorities can be absent or limited by port class, technology, law, damage, "
      "title, and local policy.\n\n"
      "Berthing and service work can consume money and game time. Depart opens the flight-plan editor; "
      "it does not launch until you review and file an executable plan. Universal managers remain "
      "available through U.\n\n"
      "Enter refreshes the port snapshot. Q asks for confirmation before returning to the BBS.",
   },
   {
      "Cargo exchange",
      "The exchange distinguishes speculative cargo you own from entrusted freight that belongs to "
      "a task or other principal. Only cargo you own can normally be sold. Capacity, lot identity, "
      "local legality, market depth, brokerage, and available cash all constrain a transaction.\n\n"
      "Market reports are dated. A quoted price can change before a remote captain arrives, and a "
      "different port may prohibit or confiscate the same goods.\n\n"
      "Buying reserves real hold capacity and credits immediately. Review tonnes, unit price, total "
      "cost, ownership, and destination before confirming.",
   },
   {
      "Fuel and supplies",
      "Refined and unrefined fuel both power jumps, but unrefined fuel may increase operational risk "
      "or affect warranty and maintenance. Port availability depends on local facilities. Frontier "
      "collection requires a charted lawful source, suitable ship capability, and game time.\n\n"
      "The route plotter can include purchases of refined or unrefined port fuel and lawful gas-giant "
      "or wilderness collection. Importing a course creates those fuel steps as real planned actions.\n\n"
      "Crew and awake passengers consume provisions. Ammunition and provisions use physical storage "
      "and must be replenished where the relevant service exists.",
   },
   {
      "Flight plans and route plotting",
      "A course may contain several jumps, in-system legs, waits, fuel purchases, and frontier fuel "
      "operations. Jump rating limits each edge. Tank capacity, current fuel, money, chart knowledge, "
      "lawful sources, and port services determine whether the whole course is executable.\n\n"
      "Fastest minimizes elapsed travel; cheapest weighs known monetary costs and may take longer. "
      "A purchased course tape uses knowledge sold at the current port and may differ from the plot "
      "available aboard.\n\n"
      "The charted-leg destination list shows distance from that leg's origin, primary-world port, "
      "population and technology codes, and charted gas giants. Open its dossier for chart age, "
      "source, and coordinates before selecting a destination.\n\n"
      "Preview shows the committed sequence. Filing is the consequential step. If a future purchase "
      "cannot be completed, the ship holds that plan rather than inventing fuel or skipping the stop.",
   },
   {
      "Banking and accounts",
      "Accounts show cash, debt, arrears, collateral, insurance or assistance, and other financial "
      "commitments recognized at this office. Recognition can be delayed when records must travel "
      "between systems.\n\n"
      "Destination assistance and similar services have explicit prices and coverage periods. "
      "Bankruptcy is an irrevocable legal process that can liquidate the fleet and create a successor "
      "career; it is not a free debt reset.\n\n"
      "Reserved collateral may still appear in your balance but cannot support another obligation.",
   },
   {
      "Shipyard and repairs",
      "A shipyard can sell vessels, accept trade-ins, replace components, refit systems, and perform "
      "proper repairs only within its local capability. Quotes depend on the exact vessel, damage, "
      "parts, title, market, and required work time.\n\n"
      "Field recovery after battle is not a substitute for proper repair. Destroyed or permanently "
      "damaged components may need replacement; temporary loss may instead be recoverable by qualified "
      "crew.\n\n"
      "A trade-in transfers a real titled vessel and its remaining stores and obligations according "
      "to the displayed terms.",
   },
   {
      "Personnel services",
      "The local exchange lists people actually available for hire and services actually present. "
      "Skills, pay, morale, condition, legal status, and berth requirements matter beyond the hiring "
      "price.\n\n"
      "The roster supports assignment, transfer, treatment, shore leave, training, and discharge. "
      "Some actions take time or require a qualified practitioner and suitable facility.\n\n"
      "Before removing or moving someone, verify that every active vessel retains a legal captain and "
      "the watchkeepers needed for its intended operation.",
   },
   {
      "Arrival checkpoint and packet",
      "The arrival packet contains messages, offers, market reports, notices, and other records made "
      "available during the voyage. I or Left ignores the current message, M or Right marks it for "
      "later, and N or Down advances to the next message. The printable keys work on terminals that "
      "cannot send arrow-key sequences. A displayed offer is still subject to physical claim and award "
      "rules.\n\n"
      "At the checkpoint the ship waits until the captain takes the arrival watch. Taking the watch "
      "can expose an encounter or complete docking. Leaving the ship holding postpones that transition.\n\n"
      "The communications receipt records which mailbags and personally carried instruments arrived.",
   },
   {
      "Voyage status",
      "Voyages advance through scheduled physical stages such as departure clearance, in-system "
      "travel, jump, arrival, and docking. The next-event time is authoritative; the real-time display "
      "is a convenience derived from the current game clock rate.\n\n"
      "Universal managers remain available during travel. Some work can proceed aboard, while port, "
      "bank, authority, personnel, and shipyard actions require the corresponding location or facility.\n\n"
      "Revising a flight plan changes future executable legs; it does not teleport the ship or undo "
      "already completed travel.",
   },
   {
      "Encounters",
      "An encounter requires a posture: fight, run, comply, surrender, or board when available. The "
      "choice affects initiative, legal consequences, objectives, and what orders can follow.\n\n"
      "Running depends on relative performance and circumstances. Complying follows the contact's "
      "demand without granting immunity from inspection or law. Surrender prioritizes survival but may "
      "transfer cargo, custody, command, or title. Boarding is close action and may expose crew directly.\n\n"
      "Review the identified contact, authority, and stakes before choosing; no posture guarantees a "
      "safe outcome.",
   },
   {
      "Vessel combat",
      "Combat resolves physical ships, people, ammunition, components, position, detection, command, "
      "and legal authority. Damage persists after battle. Ammunition expended is gone, casualties remain "
      "people, and captured cargo or vessels retain real custody and title questions.\n\n"
      "Survive, withdraw, defeat, and capture are different objectives. A standing policy tells the "
      "crew how to act when detailed orders cannot be followed, but it does not guarantee success.\n\n"
      "After action, qualified crew may perform field recovery. Proper repair, medical care, settlement, "
      "prize adjudication, and warrants can continue long after firing stops.",
   },
   {
      "Combat orders",
      "Orders assign an actor, target, weapon or system, and objective for the current activation. "
      "Only conscious, assigned, and sufficiently qualified people can perform some actions. Weapons "
      "need a functioning mount, ammunition or power, a valid target, and an applicable firing solution.\n\n"
      "Defensive, engineering, medical, maneuver, command, and boarding work compete for people and "
      "time. Recovery priority determines what the crew attempts to restore first when several systems "
      "are impaired.\n\n"
      "Review the complete joint order set before sealing it. Sealed orders are consequential for that "
      "activation and cannot be treated as a harmless preview.",
   },
}};

}  // namespace

const DoorHelp& door_help(const DoorHelpTopic topic)
{
   const auto index = static_cast<size_t>(topic);
   if(index >= HELP.size()) {
      throw std::out_of_range("invalid door help topic");
   }
   return HELP[index];
}

std::span<const DoorHelp> all_door_help()
{
   return HELP;
}

}  // namespace ct
