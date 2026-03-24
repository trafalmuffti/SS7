# SS7 Billing System

A Rust implementation of a billing and subscriber management layer for Signalling System No. 7 (SS7) networks. Includes a terminal UI served over SSH for administrative access.

## Project Goals

1. **Subscriber billing** -- Manage subscriber accounts identified by MSISDN and IMSI, track prepaid balances, and rate voice/SMS/data usage through Call Detail Records.
2. **CALEA lawful intercept** -- Provide a management interface for configuring CALEA-compliant intercept targets that instruct the signaling layer to forward subscriber packets to a designated mediation device IP.
3. **PBX call forwarding** -- Model SS7 MAP supplementary-service operations for routing VoIP and circuit-switched calls between devices (call forward unconditional, busy, no-answer, not-reachable).
4. **Operational TUI** -- Expose all management functions through a keyboard-driven terminal UI accessible over SSH with password authentication.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  ss7-tui            SSH server (russh, port 2222)   │
│  ┌───────────────────────────────────────────────┐  │
│  │  TUI (ratatui)                                │  │
│  │  Dashboard / Accounts / CALEA / PBX screens   │  │
│  └──────────────────┬────────────────────────────┘  │
└─────────────────────┼───────────────────────────────┘
                      │
┌─────────────────────┼───────────────────────────────┐
│  ss7-billing        │  Core library                 │
│  ┌──────────────────▼────────────────────────────┐  │
│  │  BillingDb (SQLite)                           │  │
│  │  ├── accounts      (MSISDN, IMSI, balance)    │  │
│  │  ├── cdrs           (call detail records)     │  │
│  │  ├── calea_intercepts (lawful intercept cfg)  │  │
│  │  └── pbx_forwarding   (call forwarding rules) │  │
│  ├───────────────────────────────────────────────┤  │
│  │  RatingEngine    CDR charge calculation       │  │
│  │  Account         Subscriber lifecycle         │  │
│  │  InterceptTarget CALEA intercept model        │  │
│  │  ForwardingRule  PBX SS7 MAP model            │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Crates

| Crate | Purpose |
|---|---|
| `ss7-billing` | Core library: data models, SQLite persistence, rating engine |
| `ss7-tui` | SSH server + terminal UI for administration |

## Running

```bash
cargo build --release
./target/release/ss7-tui

# In another terminal:
ssh root@localhost -p 2222
# Password: apple
```

The TUI presents a dashboard with six options:

| Key | Screen |
|-----|--------|
| `1` | View all subscriber accounts |
| `2` | Create a new account |
| `3` | Search accounts by name |
| `4` | Search accounts by balance range |
| `5` | CALEA intercept target management |
| `6` | PBX call forwarding rules |

## Feature Details

### Subscriber Accounts

Each account carries an MSISDN (E.164 phone number), IMSI (SIM identity), prepaid balance, and lifecycle status (active/suspended/closed). Balances are manipulated through explicit credit and debit operations with overdraft protection.

### Call Detail Records & Rating

CDRs capture per-event metadata modeled after SS7 MAP/CAP charging events:

| Field | Source |
|---|---|
| `call_type` | MO/MT voice, MO/MT SMS, data, USSD |
| `calling_party` / `called_party` | E.164 addresses from IAM or SMS-MO |
| `msc_address` | MSC Global Title from the MAP dialogue |
| `cell_id` | Cell Global Identity from the radio layer |
| `duration_seconds` | Answer/release delta from ACM/ANM/REL |

The rating engine applies configurable per-minute or flat-rate charges by call type.

### CALEA Lawful Intercept

The CALEA module manages intercept targets that instruct the SS7 signaling layer to mirror subscriber traffic to a mediation device. Each target specifies:

- **Warrant ID** -- authorization reference
- **Intercept type** -- full (voice + signaling), signaling-only, SMS, or data
- **Destination IP:port** -- address of the law-enforcement collection function

The TUI displays the equivalent SS7 operation (`MAP-TRACE`) that would be issued to the HLR. Intercepts can be activated, deactivated, or deleted.

### PBX Call Forwarding

The PBX module models SS7 MAP supplementary-service operations for VoIP/circuit-switched call routing:

| Forwarding type | SS7 MAP operation | ITU-T condition |
|---|---|---|
| Unconditional (CFU) | `RegisterSS(CFU)` | All calls forwarded immediately |
| On Busy (CFB) | `RegisterSS(CFB)` | Forward when subscriber is busy |
| No Answer (CFNA) | `RegisterSS(CFNA)` | Forward after configurable timeout |
| Not Reachable (CFNRc) | `RegisterSS(CFNRc)` | Forward when device is off-network |

Destinations can be MSISDN numbers or SIP URIs for VoIP endpoints. The TUI displays a live preview of the MAP operation that would be sent to the HLR/VLR.

## SS7 Compliance Status

This section describes how the project relates to the ITU-T Q.700-series recommendations that define Signalling System No. 7.

### What is implemented

The system implements **application-layer data models and management logic** that correspond to concepts defined in the SS7 protocol suite. It does not implement the wire protocol itself.

| SS7 Layer | Relevant specs | Status |
|---|---|---|
| **MAP subscriber data** (Q.1741, 3GPP TS 29.002) | Account model uses MSISDN and IMSI as primary subscriber identifiers, matching MAP InsertSubscriberData semantics. | Data model only |
| **MAP supplementary services** (3GPP TS 29.002 ch. 8) | PBX forwarding rules generate correct `RegisterSS` operation names for CFU, CFB, CFNA, and CFNRc with ForwardedToNumber and BasicService parameters. | Operation string generation; no ASN.1 encoding or TCAP dialogue |
| **CAP/CAMEL charging** (3GPP TS 29.078) | CDRs capture fields that would originate from CAP InitialDP and call-information-report operations (calling/called party, MSC address, cell ID, duration). The rating engine applies tariffs to these records. | Billing data model only; no real-time CAP state machine |
| **CALEA / LI** (3GPP TS 33.107, ETSI ES 201 671) | Intercept targets model the X1 provisioning interface concept (target identity, mediation device address, intercept type). | Configuration management; no X2/X3 content delivery or ETSI HI interfaces |
| **MTP / SCCP / TCAP** (Q.701-Q.714, Q.771-Q.775) | Not implemented. | -- |
| **ISUP** (Q.761-Q.764) | CDR call types (MO/MT) and party addresses correspond to ISUP IAM/ACM/ANM/REL concepts, but no ISUP message encoding exists. | Conceptual mapping only |

### What is NOT implemented

- **No wire-protocol encoding.** There is no MTP3, SCCP, TCAP, or MAP ASN.1 BER/DER codec. The system does not send or receive SS7 messages on a signaling link.
- **No signaling transport.** There is no SIGTRAN (M3UA/SCTP) or TDM interface. The SSH server is for administrative access only, not signaling.
- **No HLR/VLR/MSC integration.** MAP operations are represented as human-readable strings, not as TCAP-encoded PDUs sent to network elements.
- **No real-time call control.** There is no ISUP or CAP state machine that processes live call setup, answer, or release events.
- **No X2/X3 intercept delivery.** CALEA targets configure *where* to forward intercepted traffic, but no packet mirroring, RTP relay, or ETSI Handover Interface implementation exists.

### Roadmap toward deeper compliance

The following would be required to move from data-model compliance toward protocol-level compliance:

1. **ASN.1 codec** -- Implement BER encoding/decoding for MAP and TCAP PDUs per Q.773 and 3GPP TS 29.002 ASN.1 modules.
2. **TCAP dialogue manager** -- Handle transaction IDs, invoke/return components, and dialogue portions per Q.771-Q.775.
3. **SCCP routing** -- Implement Global Title Translation and SCCP connection-oriented/connectionless services per Q.711-Q.714.
4. **SIGTRAN transport** -- Add M3UA (RFC 4666) over SCTP for IP-based signaling transport, replacing TDM E1/T1 links.
5. **ISUP call model** -- Implement the ISUP state machine for circuit-switched call setup (IAM, ACM, ANM, REL, RLC) per Q.761-Q.764.
6. **CAP/CAMEL real-time rating** -- Implement the CAP gsmSSF/gsmSCF state machine for real-time charging decisions during call setup.
7. **LI delivery** -- Implement the ETSI HI2/HI3 or 3GPP X2/X3 interfaces for delivering intercepted content and call-associated data to the mediation device.

## Tests

```bash
cargo test
```

34 unit tests cover:

- Account CRUD, credit/debit with edge cases (insufficient funds, suspended/closed accounts)
- Search by name and balance range
- Rating engine (MO/MT calls, SMS)
- CDR storage and cascade deletion
- CALEA intercept lifecycle (create, toggle, delete, per-account filtering, active-only counting)
- PBX forwarding lifecycle (create, toggle, delete, per-account filtering, SS7 opcode generation)

## License

Public domain (Unlicense). See [LICENSE](LICENSE).
