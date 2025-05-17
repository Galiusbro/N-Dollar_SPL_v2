# Game Design Document (GDD) — CyberCats vs. CyberDogs

## 🎮 Game Overview

**Genre**: Asymmetric PvPvE Tower Defense
**Style**: Cyberpunk-futuristic with pixel/low-poly blend
**Target Platforms**: PC, Web, Mobile (cross-platform support)
**Players**: 1 Attacker (CyberDog) vs. up to 4 Defenders (CyberCats)

---

## 🧠 Core Mechanics

### 🐶 CyberDog (Attacker - Hero Role)

- Controls a **single strong unit** in real time
- Objective: **destroy the Core Node** at the center of the CyberCat base
- Gains **Bits (currency)** by:

  - Dealing damage to structures
  - Damaging the Core
  - Destroying specific buildings

- Spends Bits to buy:

  - **Gear** (armor, speed boost, advanced weapons)
  - **Active abilities** (dash, shield, EMP, drone support)

- Respawns after defeat (loses some currency)
- Each wave/phase makes the defenders stronger (encourages fast aggression)

### 🐱 CyberCats (Defenders - RTS Role)

- Each player controls a **sector** of the base
- Tasks:

  - Build and upgrade **defensive structures** (turrets, walls, traps)
  - Manage **resource buildings**
  - Coordinate **abilities** (EMP bursts, slow fields, buffs)

- Can choose roles:

  - Economy focus (building farms/miners)
  - Defense planner (walls, turret paths)
  - Support (triggering skills)

- Gain resources:

  - From time
  - From dealing damage to the CyberDog

- Can trigger **Emergency Powers**: short buffs, stuns, energy walls

---

## 🧩 Gameplay Loop

- Attacker spawns outside the base and plans infiltration
- Defender players begin with limited economy and space
- Over time:

  - Attacker gets stronger (via purchases)
  - Defenders build layered defense

- Win Conditions:

  - Dog wins if the Core is destroyed
  - Cats win if time runs out or attacker loses too many lives

---

## 🌀 Abilities

### CyberDog Abilities

- **Overdrive Dash**: Burst movement
- **MagPulse**: Temporary disables nearby turrets
- **Drone Strike**: Summons aerial bot for short assist
- **Armor Booster**: Temporary damage resistance

### CyberCat Powers

- **EMP Shock**: Stuns attacker for 2 seconds (cooldown-based)
- **Energy Wall**: Temporary barricade in any lane
- **Buff Pulse**: Boosts turret fire rate in sector
- **Trap Drop**: Instant mine spawn on the path

---

## 💰 Resource Economy

### Resource Types

- **Bits** (shared digital currency)

  - Earned from attacks (Dog) or defense/structures (Cats)

### For CyberDogs

- Earn Bits via:

  - Attacking structures
  - Damaging Core
  - Reaching milestones

- Spend Bits on:

  - Gear (speed, HP regen, attack power)
  - Abilities (see above)

### For CyberCats

- Passive income over time
- **Economic Structures**:

  - **BitMiners**: Produce Bits per second
  - **HackNodes**: Convert overcharge energy into instant Bits
  - Upgrades affect income vs. vulnerability

- Spend Bits on:

  - Turrets (Laser, Flame, EMP)
  - Traps
  - Sector buffs

---

## 🏗️ MVP Plan

1. Basic real-time hero control (CyberDog) and attack mechanics
2. Static defense building system for CyberCats
3. Single map layout with 3 lanes and core objective
4. Currency economy (earning/spending)
5. Game loop (win/loss conditions)

(More systems like progression, matchmaking, cosmetics, etc. will be added post-MVP.)
