# TOW against RTWP: function-level differences

This report compares the object-related code of the older game (`TOW/`) with the newer game (`RTWP/`), which is the base of the port. TOW levels are played under RTWP's rules, so the question answered here is which rule differences would change how a TOW level plays.

Scope: every function in `gameobject.bb`, `level.bb` and `game.bb` of both trees (menu code excluded), plus the object-related code that sits in the `game.bb` main loop (`MainGameLoop`, `UpdateGame`, input handling, per-tick update order, win and lose handling). Constants and globals that the functions depend on were compared by grepping `globals.bb` only. Rendering, HUD, menus, sound and particles are labelled cosmetic and not analysed.

## Method

1. Each file was split into functions by script. Bodies were normalised (lower-cased, comments and whitespace removed) and compared by name.
2. TOW's Stinky and Loof functions were mapped onto RTWP's player-indexed functions (Stinky is player 1, Loof is player 2 in RTWP's loader) and compared against them, not treated as TOW-only.
3. Every pair that was not identical after normalising was read and diffed by hand.
4. Pairs identical after normalising are listed compactly as "same". Identity is textual: a "same" function can still behave differently if it reads state that differs elsewhere (those cases are covered in the difference list).
5. Line numbers are 1-based and point at the `Function` line unless a quotation gives its own line.

Reading aid for status values: **same** (identical after normalising), **differs** (text differs; the summary says whether TOW's object kinds are affected), **same (indexing only)** (Stinky/Loof function equal to the RTWP player function apart from indexing and camera code), **RTWP-only** (no TOW counterpart). No function exists only in TOW once the Stinky/Loof mapping is applied.

## Counts

| | gameobject.bb | level.bb | game.bb | total |
|---|---|---|---|---|
| TOW functions in scope | 62 | 13 | 4 | 79 |
| RTWP functions in scope | 75 | 16 | 6 | 97 |
| identical after normalising (same) | 15 | 7 | 0 | 22 |
| differ (TOW rows in the tables) | 47 | 6 | 4 | 57 |
| of which Stinky/Loof functions mapped to player functions | 15 | 0 | 0 | 15 |
| TOW-only | 0 | 0 | 0 | 0 |
| RTWP-only | 20 | 3 | 2 | 25 |

Reconciliation: 79 TOW functions = 22 same + 57 differing rows. The 15 Stinky/Loof functions map onto 8 distinct RTWP functions, so 97 RTWP functions = 64 same-name counterparts + 8 mapped + 25 RTWP-only.

## gameobject.bb

| TOW function | RTWP counterpart | Status | Summary | TOW line | RTWP line |
|---|---|---|---|---|---|
| `CreateStinky` | `CreatePlayer` | differs | Same state init (position, not moving, not dying). RTWP adds PlayerExists/jump fields; camera placement, FocusOnStinky/BothCompleted reset move to LoadLevel/StartLevel. Cosmetic: shadow scale, blink counter 300 vs 370 (see D14 for RNG). | TOW/gameobject.bb:5 | RTWP/gameobject.bb:7 |
| `CanStinkyMove` | `CanPlayerMove` | differs | Same tile rules (0/2/3/8/9 and closed gates block; water/lava while moving kills), same box push/kill rules, same Loof blocking unless completed. RTWP adds: map-edge bounds check (D6), tile 17xx block, linksphere co-push, conveyor group test (D3), blocking by all players via bitmask (D13). | TOW/gameobject.bb:49 | RTWP/gameobject.bb:52 |
| `MoveStinky` | `MovePlayer` | differs | Same speed rule (BaseSpeed, halved on ice or an active conveyor), same timer/target setup. RTWP conveyor test is group aware (D3) and adds a shadow-stinker mirror move (monster types 13-20, n/a for TOW). | TOW/gameobject.bb:134 | RTWP/gameobject.bb:163 |
| `EndMoveStinky` | `EndMovePlayer` | differs | Same: breakaway step-off, button step-off, ice sliding with forward/back fallback and kill if stuck, conveyor push (kill if blocked), button step-on, teleporter, signs. RTWP adds switcher tiles, transporter, trampoline (n/a); breakaway 606 case (D17); conveyor groups and Mod 4 direction (D3); sign slots 0-4 vs 0-19 and SignActive owner (D2). | TOW/gameobject.bb:188 | RTWP/gameobject.bb:226 |
| `ControlStinky` | `ControlPlayer` | differs | Rotation smoothing only (cosmetic) - identical. RTWP adds in-flight trampoline bounds kill (n/a for TOW). | TOW/gameobject.bb:401 | RTWP/gameobject.bb:546 |
| `KillStinky` | `KillPlayer` | differs | Same dying setup (dying timer 100, position reset, switch focus if not focused). RTWP additionally ignores kills on a player who has already completed (D7), plays the death sound only for the first death (cosmetic) and forces focus to the dying player with SwitchPlayer(0) (D9). | TOW/gameobject.bb:420 | RTWP/gameobject.bb:578 |
| `CreateLoof` | `CreatePlayer` | differs | Same as CreatePlayer(2,...) for play state; sets LoofExists. Cosmetic: shadow scale/offset, blink counter 370. | TOW/gameobject.bb:449 | RTWP/gameobject.bb:7 |
| `CanLoofMove` | `CanPlayerMove` | differs | As CanStinkyMove with Stinky as the other player (no existence check needed). Differences vs RTWP are the same as for CanStinkyMove: D6, D3, D13. | TOW/gameobject.bb:493 | RTWP/gameobject.bb:52 |
| `MoveLoof` | `MovePlayer` | differs | As MoveStinky (uses LoofSpeed). Same differences: D3; mirror move n/a. | TOW/gameobject.bb:576 | RTWP/gameobject.bb:163 |
| `EndMoveLoof` | `EndMovePlayer` | differs | As EndMoveStinky (SignActive=2 for Loof; ice sound pitch 19000 vs 16000, cosmetic). RTWP differences identical to EndMoveStinky. | TOW/gameobject.bb:629 | RTWP/gameobject.bb:226 |
| `ControlLoof` | `ControlPlayer` | differs | Same as ControlStinky (cosmetic rotation); RTWP trampoline addition n/a. | TOW/gameobject.bb:839 | RTWP/gameobject.bb:546 |
| `KillLoof` | `KillPlayer` | differs | As KillStinky plus an existence guard. Same differences: D7, D9. | TOW/gameobject.bb:859 | RTWP/gameobject.bb:578 |
| `SwitchStinkyLoof` | `SwitchPlayer` | differs | Guards: needs Loof, not when the other has completed, not during first 160 ticks (LevelTimer). Sets FocusSwitchTimer=50. RTWP: SwitchPlayer cycles over 4 players, applies the 160-tick guard only when sound>0, sets FocusSwitchTimer=20 if it was below 60 (D9). | TOW/gameobject.bb:885 | RTWP/gameobject.bb:612 |
| `CreateBox` | `CreateBox` | differs | TOW builds types 0 (wood) and 1 (steel). RTWP adds types 2-12 and 100+ (push cannons) and a MaxBoxes guard (TOW has none). Types 0/1 initialise identically; BoxJumping fields n/a. | TOW/gameobject.bb:922 | RTWP/gameobject.bb:801 |
| `CanBoxMove` | `CanBoxMove` | differs | For types 0/1 the box-vs-box rules are equal (steel pushes/kills wood, same type pushes if free, wood cannot push steel, both moving: head-on kills both unless wood vs steel). Moving box kills a monster; stationary box is blocked by monsters and by items. RTWP adds map-edge check (D6), conveyor group test (D3), many type rules and ghost handling (n/a). | TOW/gameobject.bb:956 | RTWP/gameobject.bb:937 |
| `MoveBox` | `MoveBox` | differs | Same speed rule (halved on ice/active conveyor), same kill of a player or Loof/Stinky in the target tile unless moving away on a non-conveyor tile. RTWP loops over 4 players (D13) and uses a boulder push sound (cosmetic). | TOW/gameobject.bb:1093 | RTWP/gameobject.bb:1127 |
| `EndMoveBox` | `EndMoveBox` | differs | Water: wood fills water (tile becomes 100); steel makes tile 399 first, second steel fills it. Lava: wood is destroyed, steel makes 999 then fills. Ice sliding, conveyor push, button, teleporter equal. Differences: sunk box position sentinel 0 vs 101 and early Return (D10); conveyor groups (D3). RTWP adds sticky cubes, switchers, transporter, trampoline, boulders (n/a). | TOW/gameobject.bb:1151 | RTWP/gameobject.bb:1185 |
| `ControlBoxes` | `ControlBoxes` | differs | TOW: water sway animation only (cosmetic). RTWP adds trampoline bounds kill, rolling/ghost animation, boom-box proximity kill, push-cannon firing (all n/a for types 0/1). | TOW/gameobject.bb:1381 | RTWP/gameobject.bb:1527 |
| `KillBox` | `KillBox` | differs | TOW: mark dead, particles (cosmetic), step off button. RTWP adds boom-box area kill and per-type sound/particles (n/a for types 0/1; step-off-button identical). | TOW/gameobject.bb:1434 | RTWP/gameobject.bb:1677 |
| `CleanUpBoxes` | `CleanUpBoxes` | differs | Compaction identical; RTWP also copies jump and push-cannon arrays (n/a). | TOW/gameobject.bb:1452 | RTWP/gameobject.bb:1776 |
| `ControlElectroTiles` | `ControlElectroTiles` | differs | Same timing (ElectroBaseTimer, (i+2)*base off-time) and kill test (target tile if past half step, else current tile) for Stinky, Loof and monsters. RTWP loops 4 players and spares monster types 5, 6, 9 (n/a). | TOW/gameobject.bb:1545 | RTWP/gameobject.bb:2090 |
| `ControlCannons` | `ControlCannons` | differs | TOW fires when CannonTimer=0 and never limits the fireball count. RTWP fires when CannonTimer<=0 AND NofFireBalls<25 (D11). | TOW/gameobject.bb:1660 | RTWP/gameobject.bb:2288 |
| `CreateFireball` | `CreateFireball` | differs | TOW: fireball at (x,y) heading to (x+dx,y+dy). RTWP: MaxFireBalls guard (D11), reflector/prism handling at spawn (n/a), same target tile when no reflector. | TOW/gameobject.bb:1676 | RTWP/gameobject.bb:2304 |
| `ControlFireballs` | `ControlFireballs` | differs | Same movement (FireballSpeed=BaseSpeed ticks per tile). RTWP adds an edge kill at tile arrival (D11) and reflector/prism redirection (n/a); position update differs in form but equal in effect (X=X2, X2=X+d). | TOW/gameobject.bb:1710 | RTWP/gameobject.bb:2374 |
| `CanFireBallMove` | `CanFireBallMove` | differs | Same: walls/cannons/closed gates, wood box destroyed, steel box blocks, monsters die, items block, Stinky/Loof die (half-step position rule). RTWP: all 4 players, also destroys type 6 box, lets fireballs pass reflectors/prisms, spares ghost/mothership monsters (n/a). | TOW/gameobject.bb:1747 | RTWP/gameobject.bb:2439 |
| `CreateLaserGate` | `CreateLaserGate` | differs | Colours 0-4 identical; RTWP adds colours 5-7 (cosmetic/n/a). | TOW/gameobject.bb:1884 | RTWP/gameobject.bb:2572 |
| `ToggleLaserGate` | `ToggleLaserGate` | differs | Same tile logic (gate n = ((tile-1000) Mod 50)/2, +50 = open) and same kill of anything standing in a re-closing gate. RTWP kills all 4 players and spares monster type 9 (n/a). | TOW/gameobject.bb:1976 | RTWP/gameobject.bb:2673 |
| `CreateButton` | `CreateButton` | differs | Same fields. Texture atlas rows 0.125 vs 0.25 (cosmetic). The numbering of btype is interpreted differently by the gameplay functions (D3). | TOW/gameobject.bb:2054 | RTWP/gameobject.bb:2752 |
| `StepOnButton` | `StepOnButton` | differs | Same activate-if-inactive. For timer buttons (btype Mod 3 = 2) RTWP also activates and re-arms the timers of the other buttons of the same type; TOW only re-arms the stepped button (D4). | TOW/gameobject.bb:2113 | RTWP/gameobject.bb:2794 |
| `CheckButton` | `CheckButton` | differs | TOW: Stinky, Loof (alive), boxes, monsters. RTWP: all existing players (D13). Same otherwise. | TOW/gameobject.bb:2181 | RTWP/gameobject.bb:2867 |
| `ActivateButton` | `ActivateButton` | differs | TOW flips ConveyorActive (btype/3=0) or toggles gate btype/3. RTWP sets ConveyorActive(1)=False for btype/3=0, ConveyorActive(2)=False for btype/3=1 and toggles gate btype/3-1 (D3). RTWP skips self in the same-type loop. | TOW/gameobject.bb:2206 | RTWP/gameobject.bb:2894 |
| `DeActivateButton` | `DeActivateButton` | differs | Mirror of ActivateButton: TOW flips, RTWP sets ConveyorActive(1/2)=True and toggles gate btype/3-1 (D3). | TOW/gameobject.bb:2240 | RTWP/gameobject.bb:2930 |
| `StinkyStepOnTeleporter` | `PlayerStepOnTeleporter` | same (indexing only) | Compared with PlayerStepOnTeleporter: same pair lookup (MapTile-1200 match, last other found), same direction fallback order (d, -d, perpendicular, other perpendicular), same revert. Only the camera code differs (cosmetic). | TOW/gameobject.bb:2311 | RTWP/gameobject.bb:3003 |
| `LoofStepOnTeleporter` | `PlayerStepOnTeleporter` | same (indexing only) | As StinkyStepOnTeleporter; focus test inverted for Loof (camera only, cosmetic). | TOW/gameobject.bb:2371 | RTWP/gameobject.bb:3003 |
| `CreateMonster` | `CreateMonster` | differs | Types 0 (beach ball), 1/2 (robot), 3/4 (bomber): same speeds (2x, 2/3x, 3/2x BaseSpeed), same direction setup, same random draws (Rand(1,300) heading, Rand(-16,16) spin for type 0). RTWP adds types 5-20 (n/a). Mesh/animation code cosmetic. | TOW/gameobject.bb:2495 | RTWP/gameobject.bb:3133 |
| `ControlMonsters` | `ControlMonsters` | differs | TOW dispatches type 0 to BeachBall, 1/2 to Robot, everything else to Bomber. RTWP dispatches types 5-20 to their own controllers (n/a); 0-4 identical. | TOW/gameobject.bb:2635 | RTWP/gameobject.bb:3339 |
| `ControlBeachBall` | `ControlBeachBall` | differs | Movement logic, random draws and bounce identical. Player collision: TOW tests Stinky then Loof, and uses StinkySpeed/2 for Loof's half-step test (D12); RTWP loops players with each PlayerSpeed(k). | TOW/gameobject.bb:2650 | RTWP/gameobject.bb:3368 |
| `ControlRobot` | `ControlRobot` | differs | Movement logic identical. Player collision as ControlBeachBall (D12). | TOW/gameobject.bb:2757 | RTWP/gameobject.bb:3464 |
| `ControlBomber` | `ControlBomber` | differs | Movement logic and blast pattern identical. TOW uses LoofSpeed for Loof here; RTWP loops 4 players with PlayerSpeed(k). | TOW/gameobject.bb:2907 | RTWP/gameobject.bb:3833 |
| `CanMonsterMove` | `CanMonsterMove` | differs | Walkable set (floor 1xx, electro 7xx, open gate, button 11xx, sign 13xx) and water/lava for robots, boxes/monsters/items block - identical. RTWP adds map-edge check (D6) and rules for types 5-20 (n/a). | TOW/gameobject.bb:3123 | RTWP/gameobject.bb:4731 |
| `EndMoveMonster` | `EndMoveMonster` | differs | Same: step off/on button, die on water/lava. RTWP restricts the water death to types 1, 2, 11 and button handling to non-flying types (n/a; TOW kinds identical). | TOW/gameobject.bb:3201 | RTWP/gameobject.bb:4902 |
| `KillMonster` | `KillMonster` | differs | Same score (+250), same dying timer, same bomber blast (4 neighbours, kills boxes, monsters, players). RTWP loops 4 players and adds types 5, 6, 11, 12 behaviour (n/a). | TOW/gameobject.bb:3229 | RTWP/gameobject.bb:4930 |
| `CreateItem` | `CreateItem` | differs | Types 0 crystal, 1 exit gate, 2 time, 3 coin identical, except RTWP counts NofBonusCoins for type 3 and draws ItemData=Rand(360) for each type 2 item (D14, D16). | TOW/gameobject.bb:3407 | RTWP/gameobject.bb:5142 |
| `ControlItems` | `ControlItems` | differs | Pick-up rule identical (item on the target tile once the mover is past half a step and has not completed; Stinky first, then Loof). RTWP loops players with who=k (D13) and rotates time items (cosmetic). | TOW/gameobject.bb:3449 | RTWP/gameobject.bb:5187 |
| `PickUpItem` | `PickUpItem` | differs | Same: crystal count down, gate opens at 0, time +30 s, coin +250, exit: level ends when all players completed, bonus = MapTime*10, Rand(0,100) message pick. Differences: who numbering and completion bitmask (D13); RTWP progress/save handling (D16). | TOW/gameobject.bb:3525 | RTWP/gameobject.bb:5275 |
| `CleanUpItems` | `CleanUpItems` | differs | Compaction identical; RTWP also copies ItemData (D14). | TOW/gameobject.bb:3663 | RTWP/gameobject.bb:5477 |
| `CreatePillar` | `CreatePillar` | differs | TOW: pillar mesh btype 0-2, blocks its tile (ConstructLevel). RTWP: same for 0-2; adds house styles, statues and Rand(-180,180) rotation for type 22 (n/a). | TOW/gameobject.bb:3733 | RTWP/gameobject.bb:5548 |

**Same (identical after normalising):** `CreateBreakAway` (gameobject.bb:1490 / gameobject.bb:1978), `KillBreakAway` (gameobject.bb:1512 / gameobject.bb:2000), `CreateCannon` (gameobject.bb:1632 / gameobject.bb:2258), `KillFireball` (gameobject.bb:1843 / gameobject.bb:2531), `CleanUpFireballs` (gameobject.bb:1851 / gameobject.bb:2539), `StepOffButton` (gameobject.bb:2140 / gameobject.bb:2826), `CheckButtons` (gameobject.bb:2163 / gameobject.bb:2849), `CheckButtonType` (gameobject.bb:2170 / gameobject.bb:2856), `ControlTimerButtons` (gameobject.bb:2291 / gameobject.bb:2983), `BoxStepOnTeleporter` (gameobject.bb:2435 / gameobject.bb:3073), `MoveMonster` (gameobject.bb:3170 / gameobject.bb:4883), `CleanUpMonsters` (gameobject.bb:3367 / gameobject.bb:5102), `KillItem` (gameobject.bb:3657 / gameobject.bb:5471), `CreateFish` (gameobject.bb:3691 / gameobject.bb:5506), `ControlFish` (gameobject.bb:3715 / gameobject.bb:5530)

**RTWP-only functions:** `EndTrampoline` (RTWP/gameobject.bb:695), `CreateBoxer` (RTWP/gameobject.bb:1823), `ControlBoxers` (RTWP/gameobject.bb:1846), `CreateWarper` (RTWP/gameobject.bb:1899), `ControlWarpers` (RTWP/gameobject.bb:1928), `CreateSwitcher` (RTWP/gameobject.bb:2034), `KillSwitcher` (RTWP/gameobject.bb:2058), `CreateSpike` (RTWP/gameobject.bb:2169), `ControlSpikes` (RTWP/gameobject.bb:2189), `ControlSaucer` (RTWP/gameobject.bb:3606), `ControlChomper` (RTWP/gameobject.bb:4034), `ControlGhost` (RTWP/gameobject.bb:4241), `ControlSpirit` (RTWP/gameobject.bb:4350), `ControlBrokenZbot` (RTWP/gameobject.bb:4449), `ControlMotherShip` (RTWP/gameobject.bb:4518), `ControlShadowStinker` (RTWP/gameobject.bb:4577), `CreateTransporter` (RTWP/gameobject.bb:5600), `CanTransporterMove` (RTWP/gameobject.bb:5630), `MoveTransporter` (RTWP/gameobject.bb:5673), `EndMoveTransporter` (RTWP/gameobject.bb:5701)

## level.bb

| TOW function | RTWP counterpart | Status | Summary | TOW line | RTWP line |
|---|---|---|---|---|---|
| `LoadLevel` | `LoadLevel` | differs | TOW reads one fixed (version 1) format. RTWP detects the version from the header and delegates (D1). Sign slots 5 vs 20 (D2). RTWP calls no CheckButtons here (D15) and starts the focus on the first existing player. | TOW/level.bb:1 | RTWP/level.bb:1 |
| `ConstructLevel` | `ConstructLevel` | differs | Tile 104 is floor in TOW and wall in RTWP (D5). Both turn pillars into walls and restore MapTile from MapTile2. RTWP adds 5xx switchers/second conveyor group, 15xx fake walls, 16xx/22xx, 17xx/18xx tiles (n/a), more wall heights and backgrounds (cosmetic). | TOW/level.bb:210 | RTWP/level.bb:790 |
| `LevelAddWall` | `LevelAddWall` | differs | Extra height parameter in RTWP (cosmetic; TOW is fixed at 0.9). | TOW/level.bb:514 | RTWP/level.bb:1164 |
| `ShapeAddWalls` | `ShapeAddWalls` | differs | Texture coordinate scaling only (cosmetic). | TOW/level.bb:830 | RTWP/level.bb:1480 |
| `ShapeAddWallsLight` | `ShapeAddWallsLight` | differs | Wall mesh variants for heights (cosmetic). | TOW/level.bb:1094 | RTWP/level.bb:1743 |
| `ShapeAddWallsDark` | `ShapeAddWallsDark` | differs | Wall mesh variants for heights (cosmetic). | TOW/level.bb:1229 | RTWP/level.bb:1891 |

**Same (identical after normalising):** `LevelAddFloor` (level.bb:484 / level.bb:1134), `LevelAddWater` (level.bb:578 / level.bb:1228), `LevelAddLava` (level.bb:646 / level.bb:1296), `LevelAddDeepHole` (level.bb:714 / level.bb:1364), `ShapeAddFlat` (level.bb:769 / level.bb:1419), `CreateFlatSquare` (level.bb:1374 / level.bb:2049), `CreateElectro` (level.bb:1396 / level.bb:2071)

**RTWP-only functions:** `LoadLevelOldVersion` (RTWP/level.bb:183), `LoadLevelv5` (RTWP/level.bb:439), `BoxAlreadyInWater` (RTWP/level.bb:2177)

## game.bb (functions and the object-related parts of the main loop)

| TOW function | RTWP counterpart | Status | Summary | TOW line | RTWP line |
|---|---|---|---|---|---|
| `StartLevel` | `StartLevel` | differs | TOW: Stinky mesh reload, Loof reset, LoadLevel, ConstructLevel. RTWP: resets all 4 players and BothCompleted, then LoadLevel, ConstructLevel, CheckButtons and BoxAlreadyInWater (D15). Lighting, camera, recorder hooks cosmetic. | TOW/game.bb:135 | RTWP/game.bb:154 |
| `EndLevel` | `EndLevel` | differs | Resource cleanup only. RTWP also clears the new object arrays and NofBonusCoins (cosmetic for TOW). | TOW/game.bb:222 | RTWP/game.bb:316 |
| `MainGameLoop` | `MainGameLoop` | differs | Same 85 Hz tick loop (TweenPeriod=1000/85), same per-second MapTime countdown and time-out kill of every existing player, same restart/next-level hand-off. Differences are HUD (bonus coin counter), recorder hooks (cosmetic) and the level-complete delay lives in UpdateGame (D8). | TOW/game.bb:379 | RTWP/game.bb:519 |
| `UpdateGame` | `UpdateGame` | differs | Same per-tick order (input, dying/dead timers, conveyor auto-moves for mover then boxes, Control* in the same order with ControlFireballs called twice, movement translate, EndMove for players then boxes then monsters, CleanUps). Differences: input gate and switch timers (D9), conveyor groups (D3), level-complete delay 510 vs 300 ticks (D8), extra RTWP controllers (spikes, boxers, warpers, transporters; n/a). | TOW/game.bb:577 | RTWP/game.bb:733 |

**Same:** none.

**RTWP-only functions:** `NormalKeyboardControl` (RTWP/game.bb:1694), `view3dkeyboardcontrol` (RTWP/game.bb:1919)

### RTWP-only functions: reachability from TOW content

Reachable with TOW content (see the difference list): `LoadLevelOldVersion` (RTWP/level.bb:183, the version-1 path that matches TOW's object ids, D1, D2), `BoxAlreadyInWater` (RTWP/level.bb:2177, D15), `NormalKeyboardControl` (RTWP/game.bb:1694, the arrow-key input; D3, D9).

Not reachable with TOW content (object kinds or options that do not exist in TOW): `EndTrampoline`; `CreateBoxer`, `ControlBoxers`; `CreateWarper`, `ControlWarpers`; `CreateSwitcher`, `KillSwitcher`; `CreateSpike`, `ControlSpikes`; `ControlSaucer`, `ControlChomper`, `ControlGhost`, `ControlSpirit`, `ControlBrokenZbot`, `ControlMotherShip`, `ControlShadowStinker`; `CreateTransporter`, `CanTransporterMove`, `MoveTransporter`, `EndMoveTransporter`; `LoadLevelv5` (level file versions 5 and 6); `view3dkeyboardcontrol` (optional 3D camera mode).

## Differences that may affect play

Each entry gives what TOW does, what RTWP does, and the object or tile kinds touched. Entries marked "unverified" were read from the code only and not run. Each entry ends with its decision. A level question was settled by scanning all 92 TOW levels (87 in the pack, 5 bonus levels). A `Ruleset` flag is added only where a difference changes how a TOW level plays; otherwise RTWP's behaviour stands. The one flag is `cannon_fireball_limit` (D11).

### D1. Level file header, version detection and an extra field

**Touches:** every level file (loader).

**TOW** reads three strings and then the time, with no version field (`TOW/level.bb:10-13`):

```
MapName$=ReadString$(File) 	; actually jsut the "level file" message
MapName$=ReadString$(File)	; the file name (not used)
MapName$=ReadString$(File)  ; the actual description
MapTime=ReadInt(File)
```

**RTWP** decides the version from the last character of the first string (`RTWP/level.bb:57-75`: `If Right$(mapname$,1)="2"` ... `Else mapfileversion=1`) and calls `LoadLevelOldVersion` for versions below 5. That function reads an integer between the file name and the description (`RTWP/level.bb:185-190`):

```
MapName$=ReadString$(File)	; the file name (not used)
	;If mapfileversion>=3
	MapID=ReadInt(File)
	;EndIf
MapName$=ReadString$(File)  ; the actual description
```

The version guard is commented out, so it applies to version 1 as well. The shipped TOW header is "Stinky & Loof Level File" (`assets/TOW/bonus1.lev`: length-prefixed string, then `bonus1`, then `Bonus 1,` directly, no integer between them), which ends in "e" and therefore selects version 1. Read through RTWP's loader as it stands, a TOW file would consume the description's length prefix as `MapID` and misparse everything after it. RTWP's own editor guards the read with `If mapfileversion>=3` (`RTWP/editor.bb:2580`). TOW levels therefore need their own reader (or the guard restored); the object ids and tile values after the header are otherwise read the same way on the version-1 path (see D2, D3).

**Decision:** No flag. Handled: `Game::Tow` parses the header without the `MapID`, and all 87 pack levels and the 5 bonus levels load as version 1.

### D2. Object ids in the level object layer, and sign slots

**Touches:** every object kind created from the level (Stinky, Loof, boxes, items, monsters, fish, pillars, snow).

**TOW** (`TOW/level.bb:92-125`): 1 Stinky, 2 Loof, 3 wood box, 4 steel box, 5 exit gate item, 6 crystal, 7 beach ball, 8-15 robots (`CreateMonster(i,j,obj Mod 2+1,(obj-8)/2)`), 16-23 bombers (`((obj+1) Mod 2)+3,(obj-16)/2`), 24 time item, 25 coin, 26 fish, 27 snow, 28-30 pillars 0-2. Ids above 30 are ignored.

**RTWP** has three numbering schemes, chosen by file version:

| Id | TOW (and RTWP version 1, `RTWP/level.bb:290-`) | RTWP versions 2-4 (`RTWP/level.bb:355-`) | RTWP versions 5-6 (`LoadLevelv5`, `RTWP/level.bb:676-`) |
|---|---|---|---|
| 3 | wood box | crystal | player 3 |
| 4 | steel box | exit gate item | player 4 |
| 5 | exit gate item | time item | crystal |
| 6 | crystal | coin | exit gate item |
| 7 | beach ball | box type 0 (7-12 are boxes) | time item |
| 24 | time item | bomber | z-bot (18-25) |
| 25 | coin | bomber | z-bot (18-25) |

The RTWP version-1 path matches TOW for ids 1-30 (`CreateBox(i,j,0)` for 3, `CreateBox(i,j,1)` for 4, `CreateItem(i,j,1)` for 5, and so on) and adds ids 31-50 that TOW does not have. Using any other RTWP path for a TOW file changes the meaning of nearly every id.

Sign slots: TOW has 5 (`SignX(0..4)`, tile 1300-1304; `TOW/level.bb:35`), RTWP has 20 (`SignX(0..19)`). TOW's `EndMoveStinky`/`EndMoveLoof` store the sign owner as `SignActive=1` or `2` (`TOW/gameobject.bb:226`); RTWP uses the player index (`RTWP/gameobject.bb:284`, `SignActive=k`). Identical for 5 signs with two players.

**Decision:** No flag. Handled: version 1 uses this object table. The scan found no object id above 30.

### D3. Button type numbering, conveyor groups and gate index

**Touches:** button tiles (11xx), conveyor tiles (500-503), laser/iron gates (10xx), boxes/players/monsters moving on conveyors.

**TOW** has one conveyor flag. Button type `btype = tile-1100`; `btype/3=0` flips the conveyor, any other value toggles the gate whose colour index is `btype/3` (`TOW/gameobject.bb:2215-2222`, `2261-2270`; flag initialised at `TOW/level.bb:189`):

```
If ButtonType(i)/3 = 0 ; transporter
    If ConveyorActive=True ... ConveyorActive=False Else ConveyorActive=True
Else ; lasergate
    ToggleLaserGate(ButtonType(i)/3,True)
```

**RTWP** has two conveyor groups, `ConveyorActive(1)` for conveyor tiles with `Tile Mod 100 < 8` and `ConveyorActive(2)` for `>= 8`, both set True at level start (`RTWP/level.bb:140-141`). `btype/3=0` sets group 1, `btype/3=1` sets group 2, and gates use `btype/3-1` (`RTWP/gameobject.bb:2905-2912`, `2955`):

```
If ButtonType(i)/3 = 0  ... ConveyorActive(1)=False
Else If ButtonType(i)/3=1 ... ConveyorActive(2)=False
Else ToggleLaserGate(ButtonType(i)/3-1,True)
```

Activate sets, Deactivate sets the opposite, instead of flipping. For the stepped button this gives the same result as TOW's flip. The movement, push and speed tests (`CanPlayerMove`, `MovePlayer`, `EndMovePlayer`, `CanBoxMove`, `MoveBox`, `EndMoveBox`, input gating in `UpdateGame`) all test the group by `Tile Mod 100`, and direction is taken as `500+(Tile Mod 4)` (`RTWP/gameobject.bb:379-387`) instead of TOW's exact 500-503 comparison.

The numbering shift is handled in RTWP's version-1 loader, which adds 3 to button types of 3 and above (`RTWP/level.bb:243-245`):

```
If(MapTile(i,j)-1100)<3
    CreateButton(i,j,MapTile(i,j)-1100)
Else
    CreateButton(i,j,MapTile(i,j)-1100+3)
```

So TOW buttons 0-2 drive group 1 and TOW buttons 3+ drive gate `btype/3`, as in TOW. If TOW button values were passed to RTWP's functions without this remap (as the version-5 loader does, `RTWP/level.bb:550`), TOW's gate-1 buttons (types 3-5) would drive conveyor group 2, which has no tiles in TOW, and every other gate colour would be off by one. Conveyor tile values 504-511 (switchers and the second group) do not exist in TOW; for 500-503 the group and direction tests give the same answer as TOW's.

**Decision:** No flag. Handled: `Setup::load` adds 3 to version 1 button types from 3 up. For tiles 500-503 the two conveyor rules agree.

### D4. Timer buttons in a group (btype Mod 3 = 2, several buttons of one type)

**Touches:** round/timer buttons (11xx with `btype Mod 3 = 2`), gates and conveyors they drive. Unverified: derived by reading, not run.

**TOW** `StepOnButton` re-arms only the stepped button's timer (`TOW/gameobject.bb:2113-2138`). `ActivateButton(i,0)` marks every button of the same type active (`ButtonActive(j)=True`, `TOW/gameobject.bb:2228-2230`) but their `ButtonTimer` stays 0. `ControlTimerButtons` then decrements every active timer button and deactivates any at `<=0` (`TOW/gameobject.bb:2291-2305`):

```
If ButtonType(i) Mod 3 = 2 And ButtonActive(i)=True
    ButtonTimer(i)=ButtonTimer(i)-1
    If ButtonTimer(i)<=0 ... ButtonActive(i)=False : DeActivateButton(i,0)
```

On the next tick the other buttons of the group run out and `DeActivateButton` toggles the gate/conveyor back and clears the whole group, which suggests the effect of a multi-button timer group ends one tick after it starts. A single timer button of its type behaves normally.

**RTWP** activates the other buttons of the type and gives each the same timer (`RTWP/gameobject.bb:2813-2816`):

```
If ButtonType(j)=ButtonType(i) And j<>i
    ActivateButton(j,0)
    ButtonTimer(j)=ButtonBaseTimer
```

If a TOW level has two or more timer buttons of the same type, the two versions can differ visibly; whether any TOW level does should be checked against the level data.

**Decision:** No flag. No TOW level has two timer buttons of one type, so the difference is never reached.

### D5. Floor tile 104

**Touches:** tile value 104 (the fourth floor variant).

**TOW** treats 100-104 as walkable floors and draws 104 with `FloorTexture4` (`TOW/level.bb:229`, `LevelAddFloor(104,FloorTexture4)`); `CanStinkyMove` and the others only exclude tile classes 0, 2, 3, 8, 9 and closed gates.

**RTWP** converts 104 to floor only for the mesh pass and then to a wall after the tile backup is restored (`RTWP/level.bb:802`, `916`):

```
; and turn x-floors into walls
If MapTile(i,j)=104 Then MapTile(i,j)=200
```

and draws 101-103 with `FloorTexture2-4` (`RTWP/level.bb:806-808`) where TOW draws 101-104 with `FloorTexture1-4`. A TOW level that uses tile 104 gets a wall where TOW has floor. Which texture each index loads in each game was not checked.

**Decision:** No flag. No TOW level uses tile 104. Tiles 101-103 are not used either, so the texture numbering does not matter.

### D6. Map-edge bounds checks

**Touches:** players, boxes, monsters moving toward the edge of the map (tiles at x<0, y<0, x>=MapWidth, y>=MapHeight).

**TOW** has no bounds check in `CanStinkyMove`, `CanLoofMove`, `CanBoxMove`, `CanMonsterMove`; it relies on walls (or tile 0) at the border, and out-of-range `MapTile` reads are not guarded.

**RTWP** returns False first (`RTWP/gameobject.bb:61`, `949`, `4738`):

```
If x<0 Or y<0 Or x>=MapWidth Or y>=MapHeight Then Return False
```

Identical for levels whose outer ring is walls (the usual case); different only for levels with open floor on the map edge. Related: fireball edge kill (D11).

**Decision:** No flag. Only `tut4` and `play1` have anything but walls, void, water or lava on the edge. `tut4`'s are cannons pointing inward; `play1`'s are teleporters and conveyors that stay on the map. Pressing outward there meets void in TOW too.

### D7. Players who have completed the level can no longer be killed (RTWP)

**Touches:** Stinky and Loof after reaching the exit gate (item 1); any hazard that can kill a player (bombers, fireballs, boxes, electro tiles, gates, time-out).

**TOW** `KillStinky`/`KillLoof` only refuse when already dying or when the whole level is complete (`TOW/gameobject.bb:420-424`, `859-863`):

```
If StinkyDying=True Return
If LevelCompleted>0 Return
```

so a Stinky who has already finished but whose partner has not can still be killed, which starts the death sequence and a level restart.

**RTWP** `KillPlayer` also ignores a player whose completion bit is set (`RTWP/gameobject.bb:578-586`):

```
If PlayerExists(k)=True And (BothCompleted And 2^(k-1))>0
    Return
EndIf
```

This includes the time-out kill (`TOW/game.bb:502`, `RTWP/game.bb:656`). Affects two-player TOW levels where one character waits at the exit.

**Decision:** No flag. RTWP is only kinder: a waiting player cannot cost the restart, and no level can become unwinnable from it.

### D8. Level-complete delay

**Touches:** end of level after the last exit is reached (input is already locked by `LevelCompleted>0`).

**TOW:** `If LevelCompletedTimer=510 : LevelCompleted=2` (`TOW/game.bb:1366`). **RTWP:** `If LevelCompletedTimer=300 : LevelCompleted=2` (`RTWP/game.bb:1411`). Level-complete lasts 510 ticks in TOW and 300 in RTWP before the hand-off to the next level; kills during that period are ignored by both (`LevelCompleted>0`). Score, time bonus and records are computed at the moment of completion in both (D16).

**Decision:** No flag. It happens after the win, with input locked.

### D9. Switching between Stinky and Loof; focus timers

**Touches:** the Return key, Stinky and Loof control, death while unfocused, completion of the first of the two.

**TOW** (`TOW/gameobject.bb:885-896`): `SwitchStinkyLoof` returns if Loof does not exist, if the other has completed, or `If LevelTimer<160` (always), then sets `FocusSwitchTimer=50`. Input is accepted only when `FocusSwitchTimer<20` and no Stinky/Loof is dying or dead (`TOW/game.bb:720`), and a new switch needs `FocusSwitchTimer=0`. So after a switch, movement input is blocked for about 30 ticks and a new switch for 50.

**RTWP** (`RTWP/gameobject.bb:628`, `654`): `If LevelTimer<160 And sound>0 Then Return` (the guard does not apply to `SwitchPlayer(0)`), and `If FocusSwitchTimer<60 Then FocusSwitchTimer=20`. Input is blocked for about 1 tick and a new switch for 20 (the input condition is `RTWP/game.bb:841`, equivalent in meaning but for all four players). On completing the first player, RTWP sets `FocusSwitchTimer=60` then `SwitchPlayer(0)` (`RTWP/gameobject.bb:5445-5446`) where TOW calls `SwitchStinkyLoof()` (50). When the unfocused player dies, TOW calls `SwitchStinkyLoof` once and it can refuse during the first 160 ticks (`TOW/gameobject.bb:437-439`); RTWP loops `While FocusOnPlayer(k)=False : SwitchPlayer(0)` (`RTWP/gameobject.bb:602`).

Effect: the pause after a switch before the new character can move is much shorter in RTWP; affects timing-sensitive two-character puzzles and any recorded playback.

**Decision:** No flag. A shorter lock only gives the player more time, so no TOW level gets harder or unwinnable. The feel of switching does change: TOW's pause of about 30 ticks is gone.

### D10. Sunk boxes: position sentinel and early return

**Touches:** wood and steel boxes entering water or lava (and any other box leaving play).

**TOW** parks a sunk box at x=0 (`TOW/gameobject.bb:1183-1184`, `1199-1200`, `1210-1211`, `1350-1357`):

```
BoxX(i)=0
BoxX2(i)=0
```

and continues to the end of `EndMoveBox`; `BoxY` keeps its old value, so the box sits at (0, y).

**RTWP** parks it at x=101 and returns immediately (`RTWP/gameobject.bb:1259-1262`):

```
BoxX(i)=101
BoxX2(i)=101
BoxSwayingTimer(i)=0
BoxMoving(i)=False
Return
```

Which tiles and checks can see a parked box differs: at (0, y) a box could block or interact with anything at column 0 (fireball tests, `CheckButton`, `CanPlayerMove`) if that column is not a wall; at x=101 it is off any normal map. For levels with a wall border and fewer than 101 columns there is no visible difference. The water/lava results (wood fills water to tile 100; steel makes 399, then fills; lava 999) are equal.

**Decision:** No flag. Column 0 holds nothing a parked box could touch: no TOW level has a floor there.

### D11. Cannon fire rule and fireball edge handling

**Touches:** cannon tiles (8xx), fireballs, anything they hit.

**TOW** (`TOW/gameobject.bb:1660-1666`): `CannonTimer(i)=CannonTimer(i)-1 : If CannonTimer(i)=0` then fire, with no limit on live fireballs (`MaxFireballs=200`, no guard in `CreateFireball`).

**RTWP** (`RTWP/gameobject.bb:2288-2292`, `2307`, `2388`):

```
If CannonTimer(i)<=0 And NofFireBalls<25
...
If NofFireballs>=MaxFireBalls Then Return        ; in CreateFireball
...
If FireBallX2(i)<=0 Or FireBallY2(i)<=0 Or FireBallX2(i)>=Mapwidth-1 Or FireBallY2(i)>=MapHeight-1
    KillFireBall(i)
```

Effects: (a) a cannon is held back while 25 or more fireballs are alive (the timer keeps its non-positive value and the cannon fires on the first tick below the cap, then reloads), TOW never holds back; (b) RTWP kills a fireball when it arrives on a tile in the outermost ring of the map (TOW kills it only when its target tile blocks it); equal for wall borders, different for floors on the border. Fireball speed (`FireballSpeed=BaseSpeed`), reload (`(CannonSpeed+1)*CannonBaseTimer`) and the hit rules (`CanFireBallMove`) are equal for TOW's kinds.

**Decision:** **Flag added:** `Ruleset::cannon_fireball_limit`, 25 for RTWP and 200 (`MaxFireballs`) for TOW, so TOW's cannons never wait. With gates open and no boxes, `trialfire` (24 cannons) can reach 28 live fireballs and `readyrun` (26) can reach 26. The wait would shift those cannons' timing for the rest of the level. Part (b) needs no flag: no cannon shot in TOW reaches anything on the edge ring.

### D12. Monster collision with Loof uses Stinky's speed (TOW)

**Touches:** beach balls (type 0) and robots (types 1, 2) against Loof.

**TOW** tests Loof's half-step position against the wrong speed (`TOW/gameobject.bb:2715`, `2787`):

```
If LoofMoving=False Or (LoofMoving=True And LoofMovingTimer>=StinkySpeed/2)
```

(bombers use `LoofSpeed`). **RTWP** uses each player's own speed for every monster (`RTWP/gameobject.bb:2508` and others): `PlayerMovingTimer(k)>=PlayerSpeed(k)/2`. The two agree while Stinky and Loof move at the same speed; they differ while one is on ice or an active conveyor (speed `BaseSpeed/2`) and the other is not, which moves the instant at which a monster's hit switches from the old tile to the new tile for Loof.

**Decision:** No flag. This is a TOW bug that moves the hit by a few ticks in a rare case. RTWP's behaviour stands.

### D13. Order and indexing of the players in loops and `who`

**Touches:** `PickUpItem`, `ControlItems`, `CheckButton`, kills, completion tracking.

**TOW** uses `who` 0 for Stinky and 1 for Loof (`TOW/gameobject.bb:3526`), adds 1 or 2 to `BothCompleted` (`TOW/gameobject.bb:3549-3554`: "0(nobody), 1(stinky), or 2(loof) or 3(both)", `TOW/globals.bb:106`) and ends the level when `LoofExists=False Or BothCompleted=3` (`TOW/gameobject.bb:3557`). **RTWP** uses player indices 1-4, adds `2^(who-1)` (`RTWP/gameobject.bb:5297`) and ends the level when every existing player has its bit set. With two players the bit values (1 and 2) and the conditions agree; the loops still evaluate Stinky first and Loof second. Any port code that keeps TOW's 0/1 `who` while calling RTWP-style indexed functions must translate the index.

**Decision:** No flag. Same behaviour for two players; the port uses RTWP's indices.

### D14. Random draws and the RNG stream

**Touches:** the shared random stream; beach-ball movement is the only gameplay user.

Gameplay draws are identical in both versions for TOW's monsters: `CreateMonster` draws `Rand(1,300)` and `Rand(-16,16)` for each beach ball, and `ControlBeachBall` draws `dir=Rand(-1,1)` and `Rand(100)<50` (`TOW/gameobject.bb:2513`, `2589`, `2733-2735`; `RTWP/gameobject.bb:3178`, `3281`, `3440-3442`). Neither tree calls `SeedRnd` in the files searched. The streams still diverge through other draws:

- RTWP draws `ItemData(NofItems)=Rand(360)` when it creates each time item (`RTWP/gameobject.bb:5176-5178`); TOW draws nothing there.
- Cosmetic draws occur at different times: blink counter start `300` for Stinky and `370` for Loof in TOW (`TOW/gameobject.bb:36`) against `370` for every player in RTWP (`RTWP/gameobject.bb:42`); TOW's Loof idle trigger is at 1200 ticks (`TOW/game.bb:909`) and Stinky's at 900, RTWP uses 900 for all (`RTWP/game.bb:907`); RTWP draws `Rand(7,37)` for the death sound (`RTWP/game.bb:1003`).

If the port reproduces the original random stream, a TOW level with beach balls will not see the same sequence as TOW unless these draws are matched or isolated. Identical draw counts for the monsters themselves are confirmed.

**Decision:** No flag. TOW levels have no recordings to match, and the extra draws do not change what monsters can do, only which random sequence they follow.

### D15. Start-of-level order: button check, box in water

**Touches:** buttons with something standing on them at level start, boxes that start on water, player/state reset. Unverified: derived by reading, not run.

**TOW** backs up the tile map, then calls `CheckButtons()` inside `LoadLevel` (`TOW/level.bb:181-193`), and `ConstructLevel` afterwards restores `MapTile` from the backup (`TOW/level.bb:240`, `262`, `293`). A gate toggled by a button that is pressed at level start therefore appears to be reset in the tile map while its entity stays hidden, and ConveyorActive keeps the change. **RTWP** calls `CheckButtons()` after `ConstructLevel` (`RTWP/game.bb:278`), so the toggle survives, and then calls `BoxAlreadyInWater()` (`RTWP/game.bb:279`), which sinks a box that starts on a water tile (wood fills the tile to 100, any other box except type 5 makes it 399) and parks it (`RTWP/level.bb:2177-2199`). TOW has no such step. `StartLevel` also resets `PlayerExists/Moving/Dead/Dying`, focus and `BothCompleted` for four players in RTWP (`RTWP/game.bb:154-`), where TOW resets Loof state only and sets Stinky state in `CreateStinky`.

Only matters for levels that begin with a box or character on a button, or a box on water.

**Decision:** No flag. The scan found no box on water at the start, and no player or monster on a button. The only box on a button is in `return.lev`, on tile 1101, which drives conveyors, so it plays the same in both orders. Gates are never involved.

### D16. Progress and bonus-coin tracking

**Touches:** coins (item 3), level completion bookkeeping.

Score rules are the same in both (crystal opens the exit when the count reaches 0, coin +250, monster kill +250, time item +30 s, exit gives `MapTime*10`, `MapTime<1` becomes 1, `NoTimeLimit` sets it to 1). RTWP additionally counts bonus coins (`NofBonusCoins`, `RTWP/gameobject.bb:5171`, decremented on pick-up) and records a "perfect" flag when all were taken, and it calls `SavePlayer()` on every completed level where TOW saves only for `CurrentLevel<PlayGroundLevel` (`TOW/gameobject.bb:3629`; `RTWP/gameobject.bb:5440`). Custom-level (`CurrentLevel=-999`) bookkeeping exists only in RTWP. None of this changes in-level play.

**Decision:** No flag. Bookkeeping only.

### D17. Breakaway tile 606

**Touches:** breakaway floor tiles (6xx).

**TOW** treats 600 as break-to-water, 603 as break-to-lava and every other value as "reduce by one" (`TOW/gameobject.bb:207`). **RTWP** adds `OldTile=606` which breaks into nothing (tile 0) (`RTWP/gameobject.bb:265`). Tiles 600-605 behave the same; a TOW level with tile 606 (not expected) would be treated as an ordinary step-down in TOW and as a hole in RTWP.

**Decision:** No flag. No TOW level uses tile 606.

### Other points checked

- `ControlFireballs` is called twice per tick in both `UpdateGame` bodies (`TOW/game.bb:1185`, `1205`; `RTWP/game.bb:1147`, `1169`).
- Constants compared in `globals.bb`: `BaseSpeed=24`, `ElectroBaseTimer`, `CannonBaseTimer`, `ButtonBaseTimer`, `FireballSpeed` and `MaxFireballs=200` are equal; the only constant differences are array capacities (`MaxBoxes` 200 vs 400, `MaxMonsters` 200 vs 600, `MaxItems`, `MaxButtons`, and so on), `UsingPak`, and RTWP-only `SpikeBaseTimer`. Capacity matters only for levels near 200 objects of a kind (RTWP's `CreateBox` stops at `MaxBoxes`; TOW's has no guard).
- `ConveyorActive` initial value is True in both; `LaserGateActive` array sizes 5 vs 9 (more colours in RTWP).

## Compared and found equal for TOW's object kinds

Beach ball, robot and bomber behaviour (movement, turning, random draws, blast pattern, kill rules); monster speeds (2x, 2/3x and 3/2x `BaseSpeed`); `CanMonsterMove` walkable tiles; monster death scoring (+250) and bomber blast; box-versus-box rules for wood and steel; box pushing speed on ice and conveyors; ice sliding rules (forward, then back, else death); water/lava rules for boxes, robots and players; electro tile timing and kill test; teleporter pairing and fallback directions; laser gate tile logic; button step-on/step-off and `CheckButton`; the per-tick update order; the time countdown and time-out kill of every existing player; the death timers (100 ticks dying, 320 ticks dead before restart); the level intro (first 160 ticks) and the 85 Hz tick.

## Soak test

`every_tow_level_survives_random_play` in `crates/logic/tests/soak.rs` runs every TOW level with random input for up to 20,000 ticks. 300 seeds over 92 levels (27,600 runs) gave no panics: 188 won, 16,973 lost, 10,439 timed out. This shows the levels run under RTWP's rules without crashing. It does not show each level can be won: random input rarely wins, and no level was solved.

## Not verified

- D4 and D15 were read from the code and were not run in the original games; the scan shows neither is reached by a shipped level.
- The fireball peaks in D11 are upper bounds from a calculation over cannon paths. Random play reached 24 in `trialfire` and 18 in `readyrun` before the player died.
- Whether each TOW level can be finished under these rules was not tested, except by the reasoning in the decisions above.
