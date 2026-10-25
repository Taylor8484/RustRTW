
; Global Variable Definitions

; Engine Data
Const UsingPak=True

Const Version=10		; type of version - 0-9 demos, 10+ full
					;	1 - normal demo release 
					;	10 - normal full release
				

Global VersionNo1=10	; X.00
Global oldversionno1=versionno1
Const VersionNo2=0		; 0.X0
Const VersionNo3=0		; 0.0X

Const PackOverRide=1	; 1 - always use, 	-1 - never use
Const PackCleanUp=True ; delelted all tmp and x files if true

Global PackFileCount=0

Global DisplayWidth=640
Global DisplayHeight=480
Dim MyGfxModeExists(5),MyGfxModeWidth(5),MyGfxModeHeight(5)
Global MyGfxMode=1
Global MyColMode=0
Global MyWindowed=0
Global MyWindowedEd,MyColModeEd
Global OldGfxMode, OldWindowed
Global NoTimeLimit=0
Global Lockworlds=0

Global DoesSoundExist=True
Const BaseSpeed=24			; number of steps for a regular move (must be divisible by 2)

Global Music

Global Language=0	; 0 - English
					; 1 - German

; User Option Settings
Global DetailLevel=3	; 1-5


Global SecretMusic=0
; Game Variables


Global FPS, FPSCounter,FPSTime
Global ShowFPS=False
Global Tween#,TweenPeriod,TweenTime,TweenElapsed,TweenTicks
Global GamePhase=0			; 0-menu, 1-game

; Global 3D Data

Global LevelLight			; only one light used throughout game
Global CurrentLightRed,CurrentLightBlue,CurrentLightGreen
Global TargetLightRed,TargetLightGreen,TargetLightBlue
Global LightChangeSpeed
Global CurrentAmbientRed,CurrentAmbientBlue,CurrentAmbientGreen
Global TargetAmbientRed,TargetAmbientGreen,TargetAmbientBlue
Global AmbientChangeSpeed


Global Camera
Global CameraDX#,CameraDY#	; for cameraswoops: if positive, moves and reduces by step
							;					if negative, moves and adds by step
Global CameraDZ#
Global CameraDStep#=.5
Global FreeCameraX#,FreeCameraZ#


Global SaveGameStateActive=False

Dim StoryMenuPic(15)

; -----------
; LEVEL DATA
; -----------

; 1) 3D Model

Global LevelMesh
Dim LevelSurface(50) ; maximum of 50 for entire level
Global NofLevelSurfaces


Global BackGroundPlane,BackGroundPlane2,Snow, Rain, Lightning, LightningTimer, Psyche
Global LevelRed,LevelGreen,LevelBlue,LevelYellow


; 2) Game Logic

Global LevelTimer
Global CurrentLevelScore

Global MapName$
Global MapID
Global MapStyle,MapBackground
Global MapWidth, MapHeight
Global MapTime, MapMilliTime
Dim MapTile(100,100); maximum 100*100 tiles in level
Dim MapTile2(100,100); backup

Global LevelCompleted	; 0 - NO, 1-YES, in animation, 2- yes/done and go to menu
						; 3- yes/done and restart same level, 4- yes/done and start next level
Global LevelCompletedTimer
Global BothCompleted	; bits 1-4 show if player 1-4 has completed

Global BackGroundEffect=2	; 1- walls, 2- sky/forest




; Level-Textures
; --------------
; (loaded in setup)
Global WaterTexture,  IceTexture

;depending on style:
Global FloorTexture,FloorTexture2,FloorTexture3,FloorTexture4
Global FloorTextureSign
Global WallTextureTop1
Global WallTextureSide1a,WallTextureSide1b,WallTextureSide2a,WallTextureSide2b,WallTextureSide3a,WallTextureSide3b


;global for all styles:
Global IceCurveTexture1, IceCurveTexture2, IceCurveTexture3, IceCurveTexture4
Dim ConveyorTexture1(2), ConveyorTexture2(2), ConveyorTexture3(2), ConveyorTexture4(2)
Dim SwitcherTexture1(2), SwitcherTexture2(2), SwitcherTexture3(2), SwitcherTexture4(2)
Global BreakAwayTexture1, BreakAwayTexture2, BreakAwayTexture3
Global ElectroTexture
Global LavaTexture
Dim TeleportTexture(8)



Global BackGroundTexture,BackGroundTexture2

; Texture Effects
; ---------------

Global Watertexu#=0.0
Global Watertexv#=0.0
Dim conveyortex#(2)
Conveyortex#(1)=0.0
Conveyortex#(2)=0.0




; -------
; OBJECTS
; -------

; Stinky-Data
; -----------

Dim PlayerMesh(4) ; the mesh
Dim PlayerExists(4)
Dim FocusOnPlayer(4) ; true (controlling this character) or false (controlling another)
Global FocusSwitchTimer

Dim PlayerEntity(4)
Dim PlayerShadowEntity(4)
Dim PlayerTexture(4),PlayerTextureB(4),PlayerTextureC(4),PlayerTextureD(4)
Global StinkerMesh,StinkerTexture, Stinkertexture2

Dim PlayerX(4),PlayerY(4) ; current tile position
Dim PlayerX2(4),PlayerY2(4) ; tile goal (if moving)

Dim PlayerMoving(4) ; true or false
Dim PlayerMovingX(4),PlayerMovingY(4) ;-1/0/+1 depending on direction of movement
Dim PlayerMovingTimer(4) ; for how many frames will player keep moving?
Dim PlayerSpeed(4) ; the 'reset' value for MovingTimer

Dim PlayerRotationGoal(4), PlayerRotation(4) ; which way is turning/turned

Dim PlayerDying(4), PlayerDyingTimer(4)
Dim PlayerDead(4) ; true or false

Dim PlayerShrink(4)
Dim PlayerAnimCurrent(4),PlayerAnimNew(4); new is set, current is what's currently happening
	; new/current is used to switch animations already in progress, e.g. walking/pushing
							; 0 - none			(frame 1)
							; 1 - walking		(frames 2-18)
							; 2 - pushing walk	(frames 19-32)
							; 3 - dying
							; 4 - waving idle
							; 5 - tapping idle
							; 6 - bouncing idle
							; 7 - bouncing idle v2

Dim PlayerIdleCounter(4),PlayerBlinkCounter(4),PlayerBeforeBlinkTexture(4)
Dim PlayerOn(4) ; on a box in water?
Dim xsbomb(4), ysbomb(4); used For kaboom


; Box-Data	(IF UPDATED, ALSO INCLUDE IN CLEANUPBOXES())
; --------

Global WoodBoxMesh, SteelBoxMesh, GhostBoxMesh, ReflectorMesh, BoulderMesh, BoomboxMesh, PrismMesh		; the meshes 
Global WoodBoxTexture,SteelBoxTexture, GhostBoxTexture, ReflectorTexture, BoulderTexture, BoomboxTexture, PrismTexture
;Global BoxShadowTexture, BoxShadowMesh

Global NofBoxes=0
Const MaxBoxes=400

Dim BoxEntity(MaxBoxes);, BoxShadowEntity(MaxBoxes)
Dim BoxType(MaxBoxes)	; 0-wood, 1-steel, 2,3-reflector, 4-ball, 5-ghostbox, 6-boom, 7-prism

Dim BoxX(MaxBoxes),BoxY(MaxBoxes)		; current tile position
Dim BoxX2(MaxBoxes),BoxY2(MaxBoxes)		; tile goal (if moving)

Dim BoxMoving(MaxBoxes)			; true or false
Dim BoxMovingX(MaxBoxes),BoxMovingY(MaxBoxes)	; -1/0/+1 depending on direction of movement
Dim BoxMovingTimer(MaxBoxes)	; for how many frames will Stinky keep moving?
Dim BoxSpeed(MaxBoxes)			; the 'reset' value for MovingTimer
Dim BoxDead(MaxBoxes)			; true or false

Dim BoxSwayingTimer(MaxBoxes);		-1 if not swaying, otherwise through motion 0-359
Dim BoxSwayX(MaxBoxes),BoxSwayY(MaxBoxes)


; Conveyors
; ---------

Dim ConveyorActive(2)			; true or false

; Break-Away Tiles 
; ----------------

Global BreakAwayMesh

Global NofBreakAways=0
Const MaxBreakAways=400

Dim BreakAwayEntity(MaxBreakAways)
Dim BreakAwayX(MaxBreakAways),BreakAwayY(MaxBreakAways)
Dim BreakAwayType(MaxBreakAways) ; 0-almost broken, 1-2steps, 2-3steps

; Switcher Tiles 
; ----------------

Global SwitcherMesh

Global NofSwitchers=0
Const MaxSwitchers=400

Dim SwitcherEntity(MaxSwitchers)
Dim SwitcherX(MaxSwitchers),SwitcherY(MaxSwitchers)
Dim SwitcherType(MaxSwitchers) ; 0-up, 1-right, 2-down, 3-left



; Electro Tiles 
; -------------

Global ElectroMesh	; a single square

Global NofElectros=0
Const MaxElectros=400

Dim ElectroMap(100,100)

Dim ElectroEntity(4)
Dim ElectroX(MaxElectros),ElectroY(MaxElectros)
Dim ElectroExists(4)
Dim ElectroType(MaxElectros) ; 1,2,3 - (+1)*BaseTimer for timer
Dim ElectroTimer(4) ; current timer (countdown until less than basetimer- then turns on)
Const ElectroBaseTimer=BaseSpeed*2.5

; Spikes 
; -------------

Global SpikeMesh	
Global SpikeTexture

Global NofSpikes=0
Const MaxSpikes=400

Dim SpikeMap(100,100)

Dim SpikeEntity(MaxSpikes)
Dim SpikeX(MaxSpikes),SpikeY(MaxSpikes)
Dim SpikeType(MaxSpikes) ; 0,1,2,3 
Dim SpikeTimer(MaxSpikes) ; current timer (countdown until less than basetimer- then turns on)
Const SpikeBaseTimer=BaseSpeed*3


; Cannons & Fireballs
; -------------------

Global CannonMesh
Global CannonTexture

Global NofCannons=0
Const MaxCannons=400

Dim CannonEntity(MaxCannons)
Dim CannonX(MaxCannons),CannonY(MaxCannons)
Dim CannonMovingX(MaxCannons),CannonMovingY(MaxCannons)
Dim CannonSpeed(MaxCannons) ; 1,2,3 - (+1)*BaseTimer for timer
Dim CannonTimer(MaxCannons) ; current timer (countdown until less than basetimer- then turns on)
Const CannonBaseTimer=BaseSpeed*5

Global FireballMesh

Global NofFireballs=0
Const MaxFireballs=200

Dim FireballEntity(MaxFireballs)
Dim FireballX(MaxFireballs),FireballY(MaxFireballs)
Dim FireballX2(MaxFireballs),FireballY2(MaxFireballs)
Dim FireballMovingX(MaxFireballs),FireballMovingY(MaxFireballs)
Dim FireballMovingTimer(MaxFireballs)
Dim FireballDead(MaxFireballs)
Const FireballSpeed=BaseSpeed

; LaserGates
; ----------

;Global LaserGateMesh
Global IronGateMesh
;Dim LaserGateTexture(5)
Global FinalGateAnim

Global NofLaserGates=0
Const MaxLaserGates=400

Dim LaserGateEntity(MaxLaserGates), LaserGateEntity2(MaxLaserGates)
Dim LaserGateX(MaxLaserGates),LaserGateY(MaxLaserGates)
Dim LaserGateCol(MaxLaserGates)

;Dim LaserGateEntity(5)	; final (0) and one per colour 1/2/3/4
;Dim LaserGateExists(5)	; true or false if the mesh exists
Dim LaserGateActive(9)	; true or false in regards to buttons

; Buttons 
; -------------

Global ButtonOnTexture,ButtonOffTexture

Global NofButtons=0
Const MaxButtons=400

Dim ButtonEntity(MaxButtons)
Dim ButtonX(MaxButtons),ButtonY(MaxButtons)
Dim ButtonType(MaxButtons) ; 0/1/2 + Color*3
Dim ButtonActive(MaxButtons) ; True of False (as in On or Off)
Dim ButtonTimer(MaxButtons) ; current timer for timer buttons (countdown until zero - then turns off)
Const ButtonBaseTimer=BaseSpeed*15

; Monsters	
; --------

Dim MonsterMesh(12)		; the meshes (slimer, bomber,...)
Dim MonsterTexture(12)
;Dim MonsterShadowMesh(5)

Global NofMonsters=0
Const MaxMonsters=600

Dim MonsterEntity(MaxMonsters)
Dim MonsterShadowEntity(MaxMonsters)

Dim MonsterType(MaxMonsters)	; 0-coily, 1/2-zbot, 3/4-kaboom, 5/6-saucer, 7/8-chomper, 9-ghost
								; 10-rainbow spirit, 11-broken z-bot, 12-mother ship

Dim MonsterX(MaxMonsters),MonsterY(MaxMonsters)		; current tile position
Dim MonsterX2(MaxMonsters),MonsterY2(MaxMonsters)		; tile goal (if moving)

Dim MonsterMoving(MaxMonsters)			; true or false
Dim MonsterMovingX(MaxMonsters),MonsterMovingY(MaxMonsters)	; -1/0/+1 depending on direction of movement
Dim MonsterMovingTimer(MaxMonsters)	; for how many frames will Stinky keep moving?
Dim MonsterSpeed(MaxMonsters)			; the 'reset' value for MovingTimer
Dim MonsterDying(MaxMonsters)
Dim MonsterDyingTimer(MaxMonsters)
Dim MonsterDead(MaxMonsters)			; true or false

Dim MonsterData(MaxMonsters,10)			; various uses

; Fish
; -----

Global FishMesh, FishTexture
Global NofFish=0
Const MaxFish=200

Dim FishEntity(MaxFish)
Dim FishTimer#(MaxFish), FishSpeed#(MaxFish)
Dim FishX(MaxFish), FishY(MaxFish)

; Pillars
; -------
Dim PillarMesh(30), PillarTexture(30)
Global NofPillars=0
Const MaxPillars=600
Dim PillarEntity(MaxPillars)
Dim PillarType(MaxPillars)
Global HouseStyle=1
Dim HouseTexture(4)

; BoxFactory
; -------
Global BoxerMesh,BoxerTexture
Global NofBoxers=0
Const MaxBoxers=200
Dim BoxerEntity(MaxBoxers)
Dim BoxerType(MaxBoxers)
Dim BoxerTimer(MaxBoxers)
Dim BoxerX(MaxBoxers)
Dim BoxerY(MaxBoxers)









; Shadows
; -------

;Global ;RoundShadowMesh,
Global RoundShadowTexture

; Items	
; --------

Dim ItemMesh(5), ItemShadowMesh(5)		; the meshes (crystal, bonus, timer, ...)
Dim ItemTexture(5), ItemShadowTexture(5)

Global RainbowTexture,GoldStarTexture,passmaptexture

Global NofItems=0
Const MaxItems=400

Global NofCrystals=0 ; for main gate control
Global NofTotalCrystals
Global NofBonusCoins
Global NofTotalBonusCoins

Dim ItemEntity(MaxItems), ItemShadowEntity(MaxItems)
Dim ItemType(MaxItems)	; 0-crystal

Dim ItemX(MaxItems),ItemY(MaxItems)		; current tile position
Dim ItemDead(MaxItems)
Dim ItemData(MaxItems) ; currently just used as offset for clocks

; Signs
;----------

Dim SignX(20),SignY(20),SignText$(20)
Global SignActive;	0-none,1-on stinky,2-on loof (only for level-signs, i.e. #11-#30)

; Particles
; ----------

Type Particle
	Field Entity
	Field x#
	Field y#
	Field z#
	Field r#
	Field dx#
	Field dy#
	Field dz#
	Field dr#
	Field ddy#
	Field Timer
	Field Timer2
End Type

Dim ParticleMesh(5)		; various types of particles; 0-sprite, 1-flatsquare	
Global ParticleTexture, ParticleTexture2	; and textures (as animtexture, second is non-alpha)

Global NofParticles=0

; Letter Sprites
; --------------

Type LetterType
	Field Entity
	Field Tag		; 0-rainbow coins, 1-time, 2-level completed msg, 3-fps, 4-score, 5-total coins
					; 10 - levelname sign, 11-15 sign #1-#5
					; 50+ used for menus
	Field Active	; true or false

End Type


; -------------
; SOUND EFFECTS
; -------------

Global ClassicDeathSound=0

Global SFXTeleport
Global SFXDeath, SFXLevelLose, SFXButton, SFXTimerButton, SFXLevelWin, SFXHmmm, SFXTime
Global SFXLastCoin
Global SFXIce,SFXLava
Global SFXBoxPush, SFXBoxSplosh, SFXBoxExplo, SFXBreakAway, SFXBounce, SFXCoilyDeath, SFXBombExplo
Global SFXBoulderPush,SFXBoulderHit
Global SFXSaucerDie,sfxstinker
Dim SFXLightning(3)

Dim SFXPlayerConvey(4),SFXPlayerCoin(4),SFXPlayerBonus(4),SFXPlayerEnd(4),SFXPlayerClock(4)

Dim SFXTalk(20), SFXCount(10), SFXSlice(5)

Dim SFXSingle(15)	; 0-electro,1-cannonfire,2-cannonimpact
Dim SFXSingleVolume#(15)

Dim SFXMenuSound(10) ;0-menuselect,1-type



; ------------
; VARIOUS DATA
; ------------


; ------------
;     MENUS
; ------------

Global GameExit=False	; true if exit to windows
Global CurrentMenu=1
Global CurrentMenuOption
Global storymenuselected=0

Global MenuSelector; the underlying bar for selected menu option
Global MenuSelectortex

Global CursorKeyPressed, ShiftKeyPressed ; (in game, also used in end sequence)

Global Player2BDeleted
Global OptionsSoundLevel#=1.0
Global OptionsMusicLevel#=1.0
Global OptionsTeleport=1
Global OPtionsSwitchPlayerSound=1

Global MenuStyle=-1
Global MenuBkg=-1	; apparantly have nothing to do with menus, but used in MapStyle etc

Global MenuBkgStyle
Global MenuTimer#=0.0
Dim MenuEntity(80)
Dim MenuTexture(20)


; 0ld stuff - to be removed
Global MenuBackground, MenuBackGround2; Entities for planes in background
Global MenuBackgroundtex, MenuBackGroundTex2; associated textures
Global Logo1, Logo2, Logotex1,Logotex2; the Stinky& Loof Logo
; end of old stuff

                                         

; --------------
;     PLAYERS
; --------------
Dim PlayerName$(10)
Dim PlayerLevels(10)
Dim PlayerScore(10)	
Dim PlayerExtraLevels(10)
Global PlayerNameN	; how many players currently on disk (max 10)
Global CurrentPlayerName$
Global CurrentPlayerScore
Global CurrentPlayerID
Dim CurrentPlayerLevelScore(500) ; 0 means not completed, otherwise score of that level
					; 0-9 world 1, 10-19 world 2, etc. 
Dim CurrentPlayerLevelPerfect(500) ; 0 or 1
Global CurrentPlayerExtraLevels
Dim CurrentPlayerExtraLevelID(30000)
Dim CurrentPlayerExtraLevelID2(30000)
Dim CurrentPlayerExtraLevelScore(30000)
Dim CurrentPlayerExtraLevelPerfect(30000)
Global ExtraLevelMusicTrack=1


Dim FontTex(6)			; the texture anim file for 32*32 font

Global LevelNameImage,LevelNameImageBack
Global LevelNameTexture1,LevelNameTexture2
Global LevelNameEntity1,LevelNameEntity2	


; Pack File Data

Global NofPackFiles
Dim PackFilename$(1000),PackFilePosition(1000),PackFileLength(1000)

Dim pakfiles$(1000)			
Dim paklocate(1000)			
Dim PakSize(1000)			

Dim AddPack$(1000)
Dim AddPackSize(1000)
Dim AddPackOffset(1000)

Global AddPackCount
Global AddPackHeaderMask
Global AddPackKey
Global AddFileTrack

Global PackCount			;	Items in the pack file.
Global key					;	The Encrytion Key
Global Packname$			; 	The Default is DATA.PAK
Global exeoffset			;	For use when Data is packed into an .EXE
Global pakAPND$				; 	Sets up the Name to append to the start of the unpacked filenames

; WORLDS

Global CurrentWorld			; 	What world are you in now
Dim CurrentWorldPack(5)			; For the various 'packs' 0-3
Global WhichWorldPack
Global CurrentLevel, CurrentLevelBackup			; 	Level currently selected

Global CurrentWorldLevelN	; 	How many levels in this world?
Dim CurrentWorldLevelEntity(13)	; the squares for each level in the selector screen
								; up to 10 levels, + back/forward. #13 is always cursor
Global WorldLevelTexture;	a single texture containing the four symbols (no,yes,back,forward)
Global WorldLevelTexture2;	the cursor
Global WorldLevelTexture3;  perfect symbol

Dim WorldLevelName$(500)		; filenames for each level
Dim WorldName$(40,10)				; descriptorname for each world
Dim CurrentWorldLevelName$(150,3); descriptorname for each level in world (3 lines)
								;	(loaded at start of levelselectorscreen)


Global PlaygroundLevel=400	;first level of The PlayGround


; Custom Levels/Worlds

Dim FileDir$(5000)
Dim FileDir2$(5000)
Dim FileDirID(5000)
Dim FileDirID2(5000)
Dim FileDirScore(5000)
Dim FileDirPerfect(5000)
Global FileDirPos, FileDirN
Global CurrentExtraLevelDirectory$=""
Global CurrentFileDir ; the number

Global CustomSortByName=True

Global HouseDirTrue=0
Global HouseDir$=""
Global ModelDirTrue=0
Global ModelDir$=""







; ============= SECRET WORLDS STUFF ======================

;entities for the worldmap
Global WorldMap, WorldMapLevels, WorldMapCursor, StoryScreen, Worldmapshade1, Worldmapshade2

;walking
Global WorldMapWalking,WorldMapWalkingDX#,WorldMapWalkingDY#

Dim WorldMapSquare(50)
Dim WorldMapSurface(50)
Dim WorldMapTexture(50)	
Dim WorldMapTextureName$(50)

Dim WorldMapSquareX#(50)
Dim WorldMapSquareY#(50)

Dim WorldMapLevelScore(140)

Dim WorldMapLevelName$(140)
Dim WorldMapLevelStatus(140)
Dim WorldMapLevelX#(140)
Dim WorldMapLevelY#(140)
Dim WorldMapLevelExitNorth(140)
Dim WorldMapLevelExitEast(140)
Dim WorldMapLevelExitSouth(140)
Dim WorldMapLevelExitWest(140)
Dim WorldMapLevelType(140)
;ALSO CHANGE IN PLAYERCREATE!!!!!!!!!!!!!



Global CurrentMusic=-1


Global PlayerGoldStars=0
Global PlayerRainbowStars=0
Global PlayerPeeguesPass=0
Global PlayerQookiesMap=0
Global PlayerForestMap=0




Restore MapPieceInfo

n=0
Repeat
	Read WorldMapTextureName$(n)
	Read WorldMapSquareX(n)
	Read WorldMapSquareY(n)
	n=n+1
Until WorldMapTextureName$(n-1)="END"

Global NofWorldMapLevels=132; (add one (for level #0))

getworldmapinfo()









	


.MapPieceInfo
; piece, x,y
Data "mappiece1.bmp",0,0
Data "mappiece2.bmp",1,0
Data "mappiece3.bmp",2,0
Data "mappiece4.bmp",4,0
Data "mappiece5.bmp",0,1
Data "mappiece6.bmp",1,1
Data "mappiece7.bmp",2,1
Data "mappiece8.bmp",3,1
Data "mappiece9.bmp",4,1
Data "mappiece10.bmp",1.5,2
Data "mappiece11.bmp",1.5,3
Data "mappiece12.bmp",2.5,3
Data "mappiece13.bmp",3.5,2
Data "END",0,0



.MapLevelInfo
; ID,LevelName,Visible?(Y-0/N-3),Xpos,Ypos,North,East,South,West Exits,Type 
; types: 0-normal,1-secret,2-peegue,3-qookie,4-map,    
;		 10-passthrough(only two open exits)
;		 11+stories
Data 0,"easy.lv5",0,.25,.25,-1,1,1,-1,0

Data 1,"stinky1.lv5",0,22,22,-1,-1,2,-1,0
Data 2,"stinky2.lv5",0,22,28,1,3,4,4,0
Data 3,"stinky3.lv5",0,30,28,-1,-1,6,2,0
Data 4,"stinky4.lv5",0,14,32,2,2,5,-1,0
Data 5,"stinky7.lv5",0,16,42,4,6,-1,-1,0
Data 6,"stinky6.lv5",0,28,42,3,-1,130,5,0

Data 7,"gate",3,8,8,129,8,-1,-1,17
Data 8,"story",3,10,10,9,7,-1,-1,19
Data 9,"wondertown1.lv5",0,16,16,8,10,14,8,0
Data 10,"wondertown2.lv5",0,32,16,-1,11,15,9,0
Data 11,"wondertown3.lv5",0,48,16,12,12,-1,10,0
Data 12,"wondertown4.lv5",0,60,4,-1,-1,11,11,0
Data 13,"wondertown5.lv5",0,10,32,-1,14,-1,-1,2
Data 14,"wondertown6.lv5",0,18,32,9,15,16,13,0
Data 15,"wondertown7.lv5",0,32,32,10,20,17,14,0
Data 16,"wondertown10.lv5",0,16,48,14,18,19,19,0
Data 17,"wondertown9.lv5",0,32,40,15,-1,-1,-1,0
Data 18,"wondertown11.lv5",0,32,52,-1,-1,-1,16,0
Data 19,"wondertown13.lv5",3,4,60,16,16,-1,-1,1
Data 20,"gate",3,40,32,15,21,-1,-1,21
Data 21,"wondertown8.lv5",0,48,32,-1,-1,22,20,0
Data 22,"wondertown12.lv5",0,48,48,21,23,23,-1,0
Data 23,"gate",3,54,54,22,24,-1,-1,23

Data 24,"plains1.lv5",0,8,54,-1,25,31,23,0
Data 25,"plains4.lv5",0,24,48,27,26,-1,24,0
Data 26,"plains5.lv5",0,64,60,-1,-1,-1,25,0
Data 27,"plains6.lv5",0,32,24,29,28,25,-1,0
Data 28,"gate",3,47,24,35,27,-1,-1,25
Data 29,"gate",3,32,8,30,27,-1,-1,27
Data 30,"plains7.lv5",0,32,0,47,-1,29,-1,3
Data 31,"plains2.lv5",0,10,74,24,32,33,-1,0
Data 32,"plains8.lv5",3,22,74,-1,-1,-1,31,1
Data 33,"plains3.lv5",0,10,86,31,34,34,-1,0
Data 34,"gate",3,16,92,63,33,-1,-1,29

Data 35,"story",3,0,24,36,28,-1,-1,31
Data 36,"keep1.lv5",0,8,24,37,40,42,35,0
Data 37,"keep2.lv5",0,10,16,-1,38,36,-1,0
Data 38,"keep3.lv5",0,18,16,-1,39,40,37,0
Data 39,"keep4.lv5",0,26,16,-1,-1,-1,38,0
Data 40,"keep6.lv5",0,18,24,38,41,43,36,0
Data 41,"keep10.lv5",0,28,24,-1,84,44,40,0
Data 42,"keep5.lv5",0,7,32,36,43,-1,-1,0
Data 43,"keep7.lv5",0,18,32,40,-1,45,42,0
Data 44,"keep11.lv5",0,29,32,41,-1,-1,-1,0
Data 45,"keep8.lv5",0,18,39,43,46,46,-1,0
Data 46,"keep9.lv5",0,32,44,45,-1,-1,45,0

Data 47,"gate",3,64,56,49,30,-1,-1,32
Data 48,"story",3,64,52,49,47,-1,-1,34 ; no longer active
Data 49,"foggy1.lv5",0,64,48,50,-1,47,-1,0
Data 50,"foggy2.lv5",0,64,40,-1,52,49,51,0
Data 51,"foggy3.lv5",0,34,36,-1,50,-1,-1,0
Data 52,"foggy4.lv5",0,80,40,53,54,-1,50,0
Data 53,"foggy5.lv5",3,80,28,-1,-1,52,-1,1
Data 54,"foggy6.lv5",0,96,40,-1,55,-1,52,0
Data 55,"foggy7.lv5",0,118,40,56,-1,60,54,0
Data 56,"foggy8.lv5",0,118,26,57,-1,55,57,0
Data 57,"foggy9.lv5",0,106,24,58,56,56,-1,0
Data 58,"foggy10.lv5",0,112,8,-1,119,57,-1,0
Data 59,"foggy11.lv5",0,112,52,-1,60,61,-1,0
Data 60,"foggy12.lv5",0,118,52,55,-1,62,59,0
Data 61,"foggy13.lv5",0,112,58,59,62,-1,-1,0
Data 62,"foggy14.lv5",0,118,58,60,-1,-1,61,0

Data 63,"story",3,2,2,64,34,-1,-1,35
Data 64,"fireisle1.lv5",0,8,8,63,65,-1,63,0
Data 65,"fireisle2.lv5",0,24,12,66,-1,67,64,0
Data 66,"fireisle3.lv5",0,24,4,-1,-1,65,-1,0
Data 67,"fireisle4.lv5",0,24,28,65,68,-1,-1,0
Data 68,"fireisle5.lv5",0,36,28,-1,-1,69,67,0
Data 69,"fireisle6.lv5",0,36,48,68,71,-1,70,0
Data 70,"fireisle7.lv5",0,10,48,-1,69,-1,-1,0
Data 71,"fireisle8.lv5",0,64,48,72,73,-1,69,0
Data 72,"gate",3,64,44,74,71,-1,-1,36
Data 73,"fireisle9.lv5",3,92,48,-1,-1,-1,71,1
Data 74,"volcano1.lv5",0,64,40,-1,75,72,82,0
Data 75,"volcano5.lv5",0,72,40,76,76,-1,74,0
Data 76,"volcano6.lv5",0,76,34,78,77,75,-1,0
Data 77,"volcano10.lv5",3,82,34,-1,-1,-1,76,1
Data 78,"volcano7.lv5",0,72,28,-1,76,76,79,0
Data 79,"volcano8.lv5",0,64,28,-1,78,83,80,0
Data 80,"volcano4.lv5",0,56,28,-1,79,81,81,0
Data 81,"volcano3.lv5",0,52,34,80,-1,82,-1,0
Data 82,"volcano2.lv5",0,56,40,81,74,-1,81,0
Data 83,"volcano9.lv5",0,64,34,79,-1,-1,-1,4

Data 84,"forest1.lv5",0,8,24,-1,85,85,41,0
Data 85,"gate",3,12,28,86,84,-1,-1,38
Data 86,"story",3,16,32,87,85,-1,-1,40
Data 87,"forest2.lv5",0,20,36,86,88,90,86,0
Data 88,"forest3.lv5",0,36,34,89,89,92,87,0
Data 89,"forest10.lv5",3,48,16,-1,-1,88,88,1
Data 90,"forest5.lv5",0,20,44,87,92,91,91,0
Data 91,"forest6.lv5",0,8,56,90,90,-1,-1,0
Data 92,"forest4.lv5",0,40,44,88,-1,93,90,0
Data 93,"forest7.lv5",0,38,52,92,94,-1,-1,0
Data 94,"forest8.lv5",0,48,52,95,95,96,93,0
Data 95,"forest9.lv5",0,52,44,-1,-1,94,94,0
Data 96,"gate",3,48,62,97,94,-1,-1,41

Data 97,"wasteland1.lv5",0,16,6,96,98,-1,-1,0
Data 98,"wasteland2.lv5",0,28,6,-1,99,102,97,0
Data 99,"wasteland8.lv5",0,40,6,100,100,-1,98,0
Data 100,"gate",3,44,2,101,99,-1,-1,43
Data 101,"story",3,46,0,107,100,-1,-1,45
Data 102,"wasteland3.lv5",0,30,20,98,103,-1,-1,0
Data 103,"wasteland4.lv5",0,40,20,-1,-1,105,102,0
Data 104,"wasteland6.lv5",0,28,36,-1,105,-1,-1,0
Data 105,"wasteland5.lv5",0,40,36,103,106,-1,104,0
Data 106,"wasteland7.lv5",0,52,36,-1,-1,-1,105,0

Data 107,"thevoid01.lv5",0,20,48,111,108,101,101,0
Data 108,"thevoid02.lv5",0,30,48,114,109,-1,107,0
Data 109,"thevoid03.lv5",0,38,45,110,-1,-1,108,0
Data 110,"thevoid04.lv5",0,40,34,113,-1,109,113,0
Data 111,"thevoid05.lv5",0,12,37,112,112,107,-1,0
Data 112,"thevoid06.lv5",0,16,26,-1,113,111,111,0
Data 113,"thevoid07.lv5",0,32,26,-1,110,110,112,0
Data 114,"thevoid08.lv5",3,26,37,115,115,108,115,1
Data 115,"gate",3,40,20,116,113,-1,-1,46
Data 116,"final3backup.lv5",3,46,14,115,115,115,115,0
Data 117,"gate",3,48,12,118,116,-1,-1,48 ; not used
Data 118,"stinky1.lv5",3,52,8,117,-1,-1,-1,0 ; not used

Data 119,"gate",3,116-192,8,120,58,-1,-1,50
Data 120,"cloudcity1.lv5",3,12,8,-1,121,123,119,1
Data 121,"cloudcity2.lv5",3,30,14,121,-1,124,120,1
Data 122,"cloudcity4.lv5",3,55,6,-1,124,125,-1,1
Data 123,"cloudcity3.lv5",3,10,22,125,-1,-1,-1,1
Data 124,"cloudcity6.lv5",3,25,28,122,125,-1,-1,1
Data 125,"cloudcity5.lv5",3,50,24,121,-1,-1,-1,1

Data 126,"stinky11.lv5",0,24,58,130,129,127,127,0
Data 127,"stinky9.lv5",0,16,66,126,126,128,-1,0
Data 128,"stinky10.lv5",0,16,72,127,-1,-1,-1,0
Data 129,"stinky5.lv5",0,36,58,131,7,7,126,0
Data 130,"gate",3,28,50,126,6,-1,-1,52
Data 131,"stinky8.lv5",0,36,50,-1,-1,129,-1,0


Dim Storypages(50)
Dim StoryText$(50,8,5)
Dim StoryType(50) 	; 0-press enter and back to mapscreen, 
					; 1-press enter And To Next story
					; 2-press enter and continue forward (and turn off story), 
					; 3-press enter And turned back
					; 4-menu: yes-go forward, no-go back
Dim StoryGate(50)   ; type of gate: (0- not a gate) - if rejected turned into story+1
					;       1-20 # of gold, 21-40 #rainbow+20, 41-peegue, 42-qookie, 43-forest

Global CurrentStory, CurrentStoryChoice,currentpage
Dim StoryPicture$(50)
Dim storytold(20)
Restore StoryInfo
For n=0 To 46
	Read StoryPages(n)
	For i=1 To storypages(n)
		For j=1 To 5
			Read StoryText$(n,i,j)
		Next
	Next
	Read StoryType(n)
	If storytype(n)=4
		
		Read storygate(n)
	EndIf
	Read storypicture$(n)
	
	
Next

If version<10
	storypages(9)=5
	storytext$(9,4,1)="Congratulations... you have reached the end"
	storytext$(9,4,2)="of the Wonderland Secret Worlds demo."
	storytext$(9,4,3)=""
	storytext$(9,4,4)=""
	storytext$(9,4,5)=""
	storytext$(9,5,1)="But this is only the beginning of the adventure!"
	storytext$(9,5,2)="Get the full version of the game today,"
	storytext$(9,5,3)="and continue your quest to save the Stinkers."
	storytext$(9,5,5)="Don't delay... they need your help!"
	storytext$(9,5,4)=""
EndIf






.StoryInfo
;;0
Data 1,"You have discovered","a Secret Area!","","These are the hidden locations","of the mysterious Rainbow Stars!",0,"storysecret.bmp"
Data 1,"Once Upon A Time in Wonderland...","","","","",1,"storyempty.bmp"
Data 1,"After a dark and mystical night,","a magical storm blocked out morning light.","It came from the sea, out beyond Wonderland.","It blew up the water and threw up the sand.","",1,"storypic1.bmp"
Data 1,"It rattled the trees, it rattled the ground,","and made such a horrible terrible sound.","All Stinky could do was watch in dismay","as the storm swept all the little Stinkers away.","",1,"storypic2.bmp"
Data 1,"Then stillness...","      and darkness...","","What, oh what, was Stinky to do?","",1,"storyblack.bmp"
Data 1,"Then Stinky remembered a saying of Ol':","'With the help of your friends","you can reach any goal!'","So Stinky set forth, no time now to rest,","and this is where we shall begin with our quest!",0,"storypic3.bmp"
;6
Data 2,"Soon Loof joined Stinky along the way","to help a good friend save the day.","The power of two, much greater than one,","would help ensure that the task be done.",""
Data "(switch between Stinky and Loof","any time using the 'Enter' key)","","","",0,"storyloof.bmp"
;7
Data 2,"You have reached the gates of Wondertown.","","To enter, a toll of five gold stars is required.","",""
Data "Will you pay the fee?","","","","",4,5,"storystopsign.bmp"
;8
Data 1,"You have reached the gates of Wondertown.","","To enter, a toll of five gold stars is required,","but you haven't collected enough stars yet.",""
Data 3,"storystopsign.bmp"
;9
Data 3,"Standing in front of Wondertown's gate,","our heroes pondered the Stinkers' fate.","How much longer would it be","before they could help set them free?",""
Data "How many more stars would they have to earn?","How many secrets would they have to learn?","Were they indeed up to the task?","These were the questions that they did ask.",""
Data "(you can also use the 'Shift' key","and the arrow keys to see","more of the Wonderland map.)","","" ,2,"storytown.bmp"
;10
Data 2,"And so it was that Mayor Peegue","joined the Wonderland Rescue League.","Peegue's Wondertown Pass would open the way","for all three to travel to lands far away.",""
Data "(You have received Peegue's Pass.","You may use it to travel","further east into Wondertown.)","","",0,"storypeegue.bmp"
;11
Data 1,"To continue into the East Wondertown","you will need to show Mayor Peegue's Pass.","Will you show the Pass?","",""
Data 4,41,"storystopsign.bmp"
;12
Data 1,"This is a restricted area of town.","To enter you need a Pass","from Peegue, Wondertown's Mayor.","",""
Data 3,"storystopsign.bmp"
;13
Data 2,"You have reached the Wonderland exit gates.","To continue, a toll of ten gold stars is required.","","",""
Data "Will you pay the fee?","","","","",4,10,"storystopsign.bmp" 
;14
Data 1,"You have reached the Wonderland exit gates.","To continue, a toll of ten gold stars is required,","but you haven't collected enough stars yet.","",""
Data 3,"storystopsign.bmp"
;15
Data 2,"The road to the Creepy Keep is blocked.","To pass, you need to pay five gold stars.","","",""
Data "Will you pay the toll?","","","","",4,5,"storystopsign.bmp"
;16 
Data 1,"The road to the Creepy Keep is blocked.","To pass, you need to pay five gold stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;17
Data 2,"A road block prevents passage","to the Foggy Mountains.","To pass, you need to pay eight gold stars.","",""
Data "Will you pay the toll?","","","","",4,8,"storystopsign.bmp"
;18 
Data 1,"A road block prevents passage","to the Foggy Mountains.","To pass, you need to pay eight gold stars.","You don't have enough stars yet.",""
Data 3,"storystopsign.bmp"
;19
Data 2,"You have found a ship to take you to Fire Island.","Crossing the waters will cost you 15 gold stars.","","",""
Data "Will you pay the toll?","","","","",4,15,"storystopsign.bmp"
;20 
Data 1,"You have found a ship to take you to Fire Island.","Crossing the waters will cost you 15 gold stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;21
Data 2,"A gloomy castle, spooks and all,","with dungeons deep and towers tall,","and hidden dangers, little light,","and sounds and sights to give one fright!",""
Data "Eleven stars are contained in these walls,","some in the passageways, some in the halls.","Through dark corridors, secret and deep,","our heroes must now venture through Creepy Keep.","",2,"storykeep.bmp"
;22
Data 2,"You need a map to travel","into the Foggy Mountains.","","",""
Data "Use Qookie's Map?","","","","",4,42,"storystopsign.bmp"
;23 
Data 1,"You need a map to travel","into the Foggy Mountains.","It's too dangerous to go on.","",""
Data 3,"storystopsign.bmp"
;24
Data 2,"First Stinky & Loof, then Peegue made them three.","Now Qookie stood up and said 'Will you pick me?'","'YES!' was the answer, and so they were four.","Together they could accomplish oh so much more.",""
Data "To not get lost in the Mountains ahead,","Qookie took a map from the back of the shed.","In valleys below and on Foggy Peak,","these are the places where Stars they would seek.","",0,"storyqookie.bmp"
;25
Data 1,"Through wind and waves and much, much more,","our gang had reached Fire Island's shore.","A tropical paradise? Well, maybe not.","For that Volcano is awfully hot!","",2,"storyfire.bmp"
;26
Data 2,"Another road block.","To pass, you need to pay eight gold stars.","","",""
Data "Will you pay the toll?","","","","",4,8,"storystopsign.bmp"
;27 
Data 1,"Another road block.","To pass, you need to pay eight gold stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;28
Data 2,"You need a map to travel into the","Forever Forest.","","",""
Data "Use the Map from the Volcano?","","","","",4,43,"storystopsign.bmp"
;29 
Data 1,"To travel much further into this wood","a map you must find, or else it's no good.","For it you must search now, search high and low.","Here's a Hint: To a Volcano you'll go!",""
Data 3,"storystopsign.bmp"
;30
Data 1,"Snap! Rustle! What's that sound?","What's this place that we have found?","There are trees all around, and we cannot see.","Will this Forest ever set us free?","",2,"storyforest.bmp"
;31
Data 2,"The road into the Wasteland is blocked.","To pass, you need to pay 15 gold stars.","","",""
Data "Will you pay the toll?","","","","",4,15,"storystopsign.bmp"
;32 
Data 1,"The road into the Wasteland is blocked.","To pass, you need to pay 15 gold stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;33
Data 2,"To cross the chasm into the void","you must pay 10 gold stars.","","",""
Data "Will you pay the toll?","","","","",4,10,"storystopsign.bmp"
;34 
Data 1,"To cross the chasm into the void","you must pay 10 gold stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;35
Data 2,"Flash, Crack, Bang! Lightning Strikes!","There's rolling thunder that no one likes.","An empty void behind, oh my,","it's tough to keep one's spirits high.",""
Data "But this is it, in any case,","a final (?) challenge you must face.","Into the void you must now travel","and its hidden mysteries unravel.","",2,"storyvoid.bmp"
;36
Data 2,"A magical gate blocks the way ahead.","It can only be unlocked by 8 rainbow stars.","","",""
Data "Will you pay the toll?","","","","",4,28,"storystopsign.bmp"  ;change to 8 stars
;37 
Data 1,"A magical gate blocks the way ahead.","It can only be unlocked by 8 rainbow stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;38
Data 2,"A magical gate,","it can only be unlocked by 9 rainbow stars.","","",""
Data "Will you pay the toll?","","","","",4,29,"storystopsign.bmp"  ;figure out what this one was!
;39 
Data 1,"A magical gate,","it can only be unlocked by 9 rainbow stars.","You don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;40
Data 2,"What's this??? A magical road","seems to lead straight into the clouds.","It can only be unlocked by 5 rainbow stars.","",""
Data "Will you pay the toll?","","","","",4,25,"storystopsign.bmp"  
;41 
Data 1,"What's this??? A magical road","seems to lead straight into the clouds.","The gate blocking it can only be opened with","five rainbow stars. You don't have enough yet.",""
Data 3,"storystopsign.bmp"
;42
Data 2,"A toll gate is blocking your way south.","To pass, you need to pay six gold stars.","","",""
Data "Will you pay the toll?","","","","",4,6,"storystopsign.bmp"
;43 
Data 1,"A toll gate is blocking your way south.","To pass, you need to pay six gold stars,","but you don't have enough stars yet.","",""
Data 3,"storystopsign.bmp"
;44
Data 8,"Everybody now, shout 'Hooray!',","Stinky and friends have saved the day!","The Stinkers safe, the quest complete,","let no-one forget this wondrous feat.",""
Data "You did great! Want some proof?","You first met Stinky, then found Loof.","Next came Peegue and shortly after,","Qookie added to the laughter.",""
Data "You swept through Creepy Keep.","You challenged Fire Island.","You travelled to the peak","and through the mountains' highland.",""
Data "The forest was no match for you,","the wasteland was no problem, too.","Through void and city in the clouds,","you lifted every single of their shrouds.",""
Data "And thanks to four of our great thinkers","you have now saved all the Stinkers.","Super! Great! Very well done!","Congratulations! You have won!",""
Data "Congratulations!","","You have completed","Wonderland: Secret Worlds",""
Data "Thank you for playing! We hope you had fun!","Your friends at","Midnight Synergy","(you can now continue playing","to try/retry old levels)"
Data "For an extra challenge, can you","complete every last level before","returning again to save the Stinkers?","",""
Data 0,"storyend.bmp"
;45
Data 7,"Everybody now, shout 'Hooray!',","Stinky and friends have saved the day!","The Stinkers safe, the quest complete,","let no-one forget this wondrous feat.",""
Data "You did great! Want some proof?","You first met Stinky, then found Loof.","Next came Peegue and shortly after,","Qookie added to the laughter.",""
Data "You swept through Creepy Keep.","You challenged Fire Island.","You travelled to the peak","and through the mountains' highland.",""
Data "The forest was no match for you,","the wasteland was no problem, too.","Through void and city in the clouds,","you lifted every single of their shrouds.",""
Data "And thanks to four of our great thinkers","you have now saved all the Stinkers.","Super! Great! Very well done!","Congratulations! You have won!",""
Data "Congratulations!","","You have completed","Wonderland: Secret Worlds",""

Data "Thank you for playing! We hope you had fun!","Your friends at","Midnight Synergy","(you can now continue playing","to try/retry old levels)"
Data 0,"storyend.bmp"
;46
Data 7,"Everybody now, shout 'Hooray!',","Stinky and friends have saved the day!","The Stinkers safe, the quest complete,","let no-one forget this wondrous feat.",""
Data "You did great! Want some proof?","You first met Stinky, then found Loof.","Next came Peegue and shortly after,","Qookie added to the laughter.",""
Data "You swept through Creepy Keep.","You challenged Fire Island.","You travelled to the peak","and through the mountains' highland.",""
Data "The forest was no match for you,","the wasteland was no problem, too.","Through void and city in the clouds,","you lifted every single of their shrouds.",""
Data "And thanks to four of our great thinkers","you have now saved all the Stinkers.","Super! Great! Very well done!","Congratulations! You have won!",""
Data "Congratulations!","","You have completed","Wonderland: Secret Worlds",""

Data "Thank you for playing! We hope you had fun!","Your friends at","Midnight Synergy","(you can now continue playing","to try/retry old levels)"
Data 0,"storyend.bmp"































; =========================================================



Function getworldmapinfo()
	Restore maplevelinfo
	For n=0 To NofWorldMapLevels-1
		Read WorldMapID
		If n<>worldmapID
			Print "LEVEL DATA ERROR"
			Print n
			Delay 3000
			End
		EndIf
		Read WorldMapLevelName$(n)
		Read WorldMapLevelStatus(n)
		Read WorldMapLevelX(n)
		Read WorldMapLevelY(n)
		Read WorldMapLevelExitNorth(n)
		Read WorldMapLevelExitEast(n)
		Read WorldMapLevelExitSouth(n)
		Read WorldMapLevelExitWest(n)
		Read WorldMapLevelType(n)
		
		If n<7
			WorldMapLevelX(n)=WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=(WorldMapLevelY(n)+10)/32.0
		Else If n<24
			WorldMapLevelX(n)=1.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=2.0+WorldMapLevelY(n)/32.0
		Else If n<35
			WorldMapLevelX(n)=3.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=2.0+WorldMapLevelY(n)/32.0
			
		Else If n<47
			WorldMapLevelX(n)=5.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=2.0+WorldMapLevelY(n)/32.0
		Else If n<63
			WorldMapLevelX(n)=2.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=WorldMapLevelY(n)/32.0
		Else If n<84
			WorldMapLevelX(n)=4.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=6.0+WorldMapLevelY(n)/32.0
		Else If n<97
			WorldMapLevelX(n)=6.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=2.0+WorldMapLevelY(n)/32.0
		Else If n<107
			WorldMapLevelX(n)=7.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=4.0+WorldMapLevelY(n)/32.0
			
		Else If n<119
			WorldMapLevelX(n)=8.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=2.0+WorldMapLevelY(n)/32.0
		Else If n<126
			WorldMapLevelX(n)=8.0+WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=WorldMapLevelY(n)/32.0
		Else If n<132
			WorldMapLevelX(n)=WorldMapLevelX(n)/32.0
			WorldMapLevelY(n)=(WorldMapLevelY(n)+10)/32.0
		
		EndIf
		
	
	Next
End Function


.Worldlevelnames
.germantext
.frenchtext