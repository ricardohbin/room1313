# Game Design Document



## Detective Attributes & Deterioration System

### Overview

The player controls a police detective investigating increasingly disturbing cases from his desk, receiving suspects, witnesses, evidence, and reports that may involve criminal conspiracies or supernatural phenomena.

Three attributes represent his standing in society, sensitivity to the supernatural, and physiological condition. Each operates on a **0–100 scale**, with a stable middle range and two dangerous extremes.

Player decisions and external events shift these attributes. As they approach their limits, they progressively alter character interactions, available choices, evidence, and narrative events. Reaching either extreme triggers one of six tragic endings.

### Attributes

| Attribute | Low Extreme | Balanced | High Extreme |
|---|---|---|---|
| **Reputation** | Disgrace | Credibility | Notoriety |
| **Sensitivity** | Spiritual Blindness | Attunement | Revelation / Psychosis |
| **Arousal** | Exhaustion | Composure | Hyperstimulation |

### 1. Reputation — Social Standing

Measures how the detective is perceived by colleagues, authorities, suspects, the public, and criminal organizations.

**Low — Disgrace**
- **Unease:** Witnesses become uncooperative, colleagues question his judgment, and rumors undermine his credibility.
- **Deterioration:** Access to evidence and institutional resources becomes restricted. Superiors interfere, and suspects openly disregard his authority.
- **Crisis:** Internal Affairs investigates, evidence is confiscated, and suspension becomes imminent.
- **Loss — Dismissal:** The detective is stripped of his badge, permanently removed from the investigation, and left powerless to expose the truth.

**High — Notoriety**
- **Unease:** Recognition brings important cases, unsolicited information, and influential contacts.
- **Deterioration:** Public scrutiny intensifies. Criminal organizations monitor his movements, informants disappear, and threats arrive.
- **Crisis:** Bribery, blackmail, compromised colleagues, and assassination attempts threaten his survival.
- **Loss — Assassination:** The criminal organization has the detective murdered before he can expose its secrets.

### 2. Sensitivity — Psychological & Supernatural Awareness

Measures the detective's receptiveness to supernatural phenomena, inseparable from his psychological condition. Too little leaves him defenseless against unseen influences; too much overwhelms his perception of reality.

**Low — Spiritual Blindness**
- **Unease:** Emotional numbness and apathy obscure disturbing details. Spiritual clues appear ordinary or go unnoticed.
- **Deterioration:** Unexplained footsteps, missing time, altered evidence, and invisible presences disturb him. He becomes increasingly frightened by things he cannot perceive or understand.
- **Crisis:** Unseen entities manipulate events directly, creating dangerous situations and driving him into desperate reactions.
- **Loss — Fatal Haunting:** Terrified and pursued by invisible forces, the detective suffers a fatal accident while attempting to escape.

**High — Revelation / Psychosis**
- **Unease:** Hidden figures emerge in photographs, voices become audible in recordings, and occult connections reveal new investigative possibilities.
- **Deterioration:** Supernatural manifestations invade ordinary perception. Suspects appear transformed, evidence speaks, and the detective struggles to distinguish real events from visions.
- **Crisis:** He responds to voices and entities nobody else perceives. His erratic behavior alarms colleagues and compromises the investigation.
- **Loss — Institutionalization:** Following a severe psychological crisis, the detective is involuntarily committed to a psychiatric institution. He may have uncovered the truth, but nobody believes him.

### 3. Arousal — Physiological Activation

Measures the detective's nervous system activation, ranging from profound exhaustion to overwhelming hyperstimulation.

**Low — Exhaustion**
- **Unease:** Fatigue causes sluggish responses, difficulty concentrating, and occasional missed details.
- **Deterioration:** Microsleeps and blackouts interrupt interrogations. Time passes unnoticed, testimonies are lost, and unexplained actions appear in his records.
- **Crisis:** Severe exhaustion causes frequent lapses in consciousness, physical weakness, and an inability to remain attentive.
- **Loss — Coma:** The detective collapses at his desk and is hospitalized in a coma, permanently ending his involvement in the case.

**High — Hyperstimulation**
- **Unease:** Restlessness, impatience, and heightened responsiveness make interruptions increasingly difficult to ignore.
- **Deterioration:** Ringing telephones, overlapping testimonies, urgent reports, and sensory disturbances fragment his attention. Impulsive decisions and physical symptoms escalate.
- **Crisis:** Severe agitation, trembling, palpitations, and chest pain accompany an overwhelming succession of demands.
- **Loss — Fatal Overload:** The detective suffers a fatal heart attack or stroke, overwhelmed by sustained physiological stress.

### Deterioration Thresholds

| Attribute Value | State | Gameplay Effect |
|---|---|---|
| 40–60 | **Balanced** | Normal interactions and card distribution |
| 20–39 / 61–80 | **Unease** | Altered dialogue, early symptoms, occasional special events |
| 1–19 / 81–99 | **Crisis** | Severe symptoms, frequent dangerous encounters, imminent loss |
| 0 / 100 | **Loss** | Attribute-specific tragic ending |

Deterioration is progressive. Each stage changes the narrative and introduces new opportunities, dangers, and decisions. The player receives recognizable warnings before reaching a fatal threshold.

### Card & Narrative Integration

Attributes influence the game's events through a shared card system rather than separate mechanical subsystems.

- **Card selection:** Current conditions influence which suspects, reports, interruptions, and supernatural events appear.
- **Contextual variations:** Existing cards change their descriptions, dialogue, and consequences according to attribute states.
- **Available choices:** Certain investigative approaches appear, disappear, or become dangerous depending on the detective's condition.
- **Cross-attribute effects:** Events can affect multiple attributes, generating difficult tradeoffs.

**Examples:**
- High Reputation attracts important witnesses but also surveillance and threats from organized crime.
- Low Reputation restricts official investigative resources but makes the detective less conspicuous.
- High Sensitivity reveals supernatural clues while introducing disturbing and potentially misleading perceptions.
- Low Sensitivity conceals supernatural threats, allowing entities to manipulate events without the detective realizing it.
- Low Arousal causes lost time, missed testimony, and involuntary lapses in attention.
- High Arousal introduces interruptions, competing demands, and increasingly impulsive reactions.

### Design Principles

1. **Both extremes are dangerous.** No attribute should be treated as a conventional resource that is always beneficial to increase.
2. **Instability provides opportunities.** Moving away from balance may unlock useful information, relationships, or investigative approaches before becoming destructive.
3. **Consequences escalate visibly.** Dialogue, character behavior, environmental details, and card events foreshadow the corresponding loss condition.
4. **Deterioration creates dilemmas.** Dangerous states introduce recovery opportunities that may compromise other attributes, rather than simply removing player agency.
5. **Attributes remain distinct.** Reputation governs social and institutional relationships; Sensitivity governs psychological and supernatural perception; Arousal governs physiological responsiveness.
6. **Losses are narrative tragedies.** Each extreme concludes the investigation through a distinct neo-noir fate, expressed through a short epilogue, police report, newspaper clipping, or hospital record.

### Core Design Goal

The player must balance **credibility, supernatural awareness, and physiological endurance** while pursuing an investigation that progressively destabilizes all three.

The central dilemma is not simply whether the detective can uncover the truth, but **how much of himself he can sacrifice in the pursuit of it without losing the ability to finish the case.**




## Case 358 — Room 1313

### Setting

The investigation centers on a mysterious crime committed in **Room 1313**. The case is officially registered as **Case 358**, a seemingly ordinary identification number concealing an occult significance.

### Esoteric Symbolism

In Hebrew Gematria, letters carry numerical values, allowing different words to share symbolic associations through their numerical equivalence.

Two significant Hebrew words share the value **358**:

| Hebrew | Transliteration | Meaning | Gematria |
|---|---|---|---|
| נחש | *Nachash* | Serpent | 50 + 8 + 300 = 358 |
| משיח | *Mashiach* | Messiah | 40 + 300 + 10 + 8 = 358 |

This traditional numerical correspondence associates two contrasting symbols: **the Serpent and the Messiah**, often interpreted through themes of temptation, corruption, salvation, and transformation.

The case number therefore suggests an unsettling ambiguity: **is the force behind the crime a destroyer or a savior?**

### Room 1313

The repetition of thirteen reinforces the game's *Unlucky 13* theme. Its symbolic association with misfortune contrasts with the promise of salvation implicit in Case 358.

Room 1313 serves as the central mystery, potentially existing at the boundary between ordinary reality and a concealed supernatural world.

No established historical Gematria correspondence between 1313 and 358 is assumed. Their relationship belongs to the game's fictional occult mythology.

### Narrative Integration

- **Initial investigation:** Case 358 concerns an apparently ordinary crime in Room 1313.
- **Emerging connections:** Evidence introduces recurring serpentine imagery, religious references, and unexplained numerical coincidences.
- **Occult revelation:** The detective discovers the Gematria correspondence between *Nachash* and *Mashiach*, suggesting that the same entity may be interpreted as either a corrupting influence or a figure of salvation.
- **Central ambiguity:** Witnesses, suspects, and supernatural manifestations provide conflicting accounts of the force associated with the room. Its true nature remains uncertain.

### Design Intent

The numbers should function as **discoverable clues rather than explicit exposition**. Their significance emerges through documents, testimony, artifacts, and the detective's changing Sensitivity.

The underlying horror comes from the possibility that salvation and damnation are not opposites, but different interpretations of the same supernatural event.