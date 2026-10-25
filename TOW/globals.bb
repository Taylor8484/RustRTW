
; Global Variable Definitions

; Engine Data
Const UsingPak=False

Const Version=10		; type of version - 0-9 demos, 10+ full
					;	0,1 - normal demo release (1 was used as of v1.08)
					
					; 	2-9 Various Affiliate Versions (#s change, recently just used #2 for all)
					; 	these are old ones:
						;   2 - Joerg Mad Data
						;   3 - Weasel Circus
						; 	4 - General Affiliate 4 (Currently BrainGamez)
						;   5 - General Affiliate 5 (Currently Games4Win)
						;   6 - General Affiliate 6 (Currently Search Amateur)
						; 	7 - Five Star Network Affiliate Release
						;   8 - Download-Free-Games Affiliate Release
						;	9 - Coffeebreakarcade Affiliate Release
					;	10 - normal full release
					;	11 - press demo full release (extra menu)
					; 	12 - astragon (World 8 and 9 disabled)

Global VersionNo1=10	; X.00
Const VersionNo2=0		; 0.X0
Const VersionNo3=0		; 0.0X

Const PackOverRide=1	; 1 - always use, 	-1 - never use
Const PackCleanUp=True ; delelted all tmp and x files if true

Global PackFileCount=0

Global DisplayWidth=640
Global DisplayHeight=480
Dim MyGfxModeExists(5),MyGfxModeWidth(5),MyGfxModeHeight(5)
Global MyGfxMode=0
Global MyColMode=1;
Global MyWindowed=0
Global OldGfxMode, OldWindowed
Global NoTimeLimit=0
Global LockWorlds=0;

Global EndMenuStage=0
Global IsExtraDemoLevelActive=0

Global DoesSoundExist=True
Const BaseSpeed=24			; number of steps for a regular move (must be divisible by 2)

Global Music

Global Language=0	; 0 - English
					; 1 - German

; User Option Settings
Global DetailLevel=5	; 1-5


; Game Variables


Global FPS, FPSCounter,FPSTime
Global ShowFPS=False
Global Tween#,TweenPeriod,TweenTime,TweenElapsed,TweenTicks
Global GamePhase=0			; 0-menu, 1-game

; Global 3D Data

Global LevelLight			; only one light used throughout game
Global Camera
Global CameraDX#,CameraDY#	; for cameraswoops: if positive, moves and reduces by step
							;					if negative, moves and adds by step
Global CameraDZ#
Global CameraDStep#=.5
Global FreeCameraX#,FreeCameraZ#


; -----------
; LEVEL DATA
; -----------

; 1) 3D Model

Global LevelMesh
Dim LevelSurface(50) ; maximum of 50 for entire level
Global NofLevelSurfaces


Global BackGroundPlane,BackGroundPlane2,Snow, Rain


; 2) Game Logic

Global LevelTimer
Global CurrentLevelScore

Global MapName$
Global MapStyle,MapBackground
Global MapWidth, MapHeight
Global MapTime, MapMilliTime
Dim MapTile(100,100); maximum 100*100 tiles in level
Dim MapTile2(100,100); backup

Global LevelCompleted	; 0 - NO, 1-YES, in animation, 2- yes/done and go to menu
						; 3- yes/done and restart same level, 4- yes/done and start next level
Global LevelCompletedTimer
Global BothCompleted	; 0(nobody), 1(stinky), or 2(loof) or 3(both)

Global BackGroundEffect=2	; 1- walls, 2- sky/forest




; Level-Textures
; --------------
; (loaded in setup)
Global WaterTexture,  IceTexture

;depending on style:
Global FloorTexture
Global FloorTextureSign
Global WallTextureTop1
Global WallTextureSide1a,WallTextureSide1b,WallTextureSide2a,WallTextureSide2b

;global for all styles:
Global IceCurveTexture1, IceCurveTexture2, IceCurveTexture3, IceCurveTexture4
Global ConveyorTexture1, ConveyorTexture2, ConveyorTexture3, ConveyorTexture4
Global BreakAwayTexture1, BreakAwayTexture2, BreakAwayTexture3
Global ElectroTexture
Global LavaTexture
Dim TeleportTexture(4)



Global BackGroundTexture

; Texture Effects
; ---------------

Global Watertexu#=0.0
Global Watertexv#=0.0
Global conveyortex#=0.0




; -------
; OBJECTS
; -------

; Stinky-Data
; -----------
Global StinkyMesh	; the mesh

Global FocusOnStinky	; true (controlling stinky) or false (controlling loof)
Global FocusSwitchTimer ; counts down to zero until next switch can be made

Global StinkyEntity		; the (only) entity
Global StinkyShadowEntity
Global StinkyTexture, StinkyTextureB, StinkyTextureC, StinkyTextureD
;Global StinkyWarpSphere	; used at beginning of level
	
Global StinkyX,StinkyY		; current tile position
Global StinkyX2,StinkyY2		; tile goal (if moving)

Global StinkyMoving			; true or false
Global StinkyMovingX,StinkyMovingY ; -1/0/+1 depending on direction of movement
Global StinkyMovingTimer	; for how many frames will Stinky keep moving?
Global StinkySpeed			; the 'reset' value for MovingTimer

Global StinkyRotationGoal, StinkyRotation  ; which way is it turning/turned

Global StinkyDying
Global StinkyDyingTimer
Global StinkyDead 			; true or false

Global StinkyShrink
Global StinkyAnimCurrent, StinkyAnimNew ; new is set, current is what's currently happening
	; new/current is used to switch animations already in progress, e.g. walking/pushing
							; 0 - none			(frame 1)
							; 1 - walking		(frames 2-18)
							; 2 - pushing walk	(frames 19-32)
							; 3 - dying
							; 4 - waving idle
							; 5 - tapping idle
							; 6 - bouncing idle
							; 7 - bouncing idle v2
Global StinkyIdleCounter,StinkyBlinkCounter,StinkyBeforeBlinkTexture														
;Global StinkyLastNoX,StinkyLastNoY,StinkyLastNoDX,StinkyLastNoDY

							

; Loof-Data
; -----------

Global LoofExists	;true or false, since not always true

Global LoofMesh	; the mesh

Global LoofEntity		; the (only) entity
Global LoofShadowEntity
Global LoofTexture, LoofTextureB, LoofTextureC, LoofTextureD
;Global LoofWarpSphere	; used at beginning of level
	
Global LoofX,LoofY		; current tile position
Global LoofX2,LoofY2		; tile goal (if moving)

Global LoofMoving			; true or false
Global LoofMovingX,LoofMovingY ; -1/0/+1 depending on direction of movement
Global LoofMovingTimer	; for how many frames will Loof keep moving?
Global LoofSpeed			; the 'reset' value for MovingTimer

Global LoofRotationGoal, LoofRotation ; which way is it turning/turned

Global LoofDying
Global LoofDyingTimer
Global LoofDead 			; true or false

Global LoofShrink
Global LoofAnimCurrent, LoofAnimNew
Global LoofIdleCounter,LoofBlinkCounter,LoofBeforeBlinkTexture

; Box-Data	(IF UPDATED, ALSO INCLUDE IN CLEANUPBOXES())
; --------

Global WoodBoxMesh, SteelBoxMesh		; the meshes 
Global WoodBoxTexture,SteelBoxTexture
;Global BoxShadowTexture, BoxShadowMesh

Global NofBoxes=0
Const MaxBoxes=200

Dim BoxEntity(MaxBoxes);, BoxShadowEntity(MaxBoxes)
Dim BoxType(MaxBoxes)	; 0-wood, 1-steel

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

Global ConveyorActive			; true or false

; Break-Away Tiles 
; ----------------

Global BreakAwayMesh

Global NofBreakAways=0
Const MaxBreakAways=200

Dim BreakAwayEntity(MaxBreakAways)
Dim BreakAwayX(MaxBreakAways),BreakAwayY(MaxBreakAways)
Dim BreakAwayType(MaxBreakAways) ; 0-almost broken, 1-2steps, 2-3steps


; Electro Tiles 
; -------------

Global ElectroMesh	; a single square

Global NofElectros=0
Const MaxElectros=200

Dim ElectroMap(100,100)

Dim ElectroEntity(4)
Dim ElectroX(MaxElectros),ElectroY(MaxElectros)
Dim ElectroExists(4)
Dim ElectroType(MaxElectros) ; 1,2,3 - (+1)*BaseTimer for timer
Dim ElectroTimer(4) ; current timer (countdown until less than basetimer- then turns on)
Const ElectroBaseTimer=BaseSpeed*2.5

; Cannons & Fireballs
; -------------------

Global CannonMesh
Global CannonTexture

Global NofCannons=0
Const MaxCannons=200

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
Const MaxLaserGates=200

Dim LaserGateEntity(MaxLaserGates), LaserGateEntity2(MaxLaserGates)
Dim LaserGateX(MaxLaserGates),LaserGateY(MaxLaserGates)
Dim LaserGateCol(MaxLaserGates)

;Dim LaserGateEntity(5)	; final (0) and one per colour 1/2/3/4
;Dim LaserGateExists(5)	; true or false if the mesh exists
Dim LaserGateActive(5)	; true or false in regards to buttons

; Buttons 
; -------------

Global ButtonOnTexture,ButtonOffTexture

Global NofButtons=0
Const MaxButtons=200

Dim ButtonEntity(MaxButtons)
Dim ButtonX(MaxButtons),ButtonY(MaxButtons)
Dim ButtonType(MaxButtons) ; 0/1/2 + Color*3
Dim ButtonActive(MaxButtons) ; True of False (as in On or Off)
Dim ButtonTimer(MaxButtons) ; current timer for timer buttons (countdown until zero - then turns off)
Const ButtonBaseTimer=BaseSpeed*15

; Monsters	
; --------

Dim MonsterMesh(5)		; the meshes (slimer, bomber,...)
Dim MonsterTexture(5)
;Dim MonsterShadowMesh(5)

Global NofMonsters=0
Const MaxMonsters=200

Dim MonsterEntity(MaxMonsters)
Dim MonsterShadowEntity(MaxMonsters)

Dim MonsterType(MaxMonsters)	; 0-slimer, 1-bomber

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
Dim PillarMesh(5), PillarTexture(5)
Global NofPillars=0
Const MaxPillars=200
Dim PillarEntity(MaxPillars)



; Shadows
; -------

;Global ;RoundShadowMesh,
Global RoundShadowTexture

; Items	
; --------

Dim ItemMesh(5), ItemShadowMesh(5)		; the meshes (crystal, bonus, timer, ...)
Dim ItemTexture(5), ItemShadowTexture(5)

Global RainbowTexture

Global NofItems=0
Const MaxItems=200

Global NofCrystals=0 ; for main gate control
Global NofTotalCrystals

Dim ItemEntity(MaxItems), ItemShadowEntity(MaxItems)
Dim ItemType(MaxItems)	; 0-crystal

Dim ItemX(MaxItems),ItemY(MaxItems)		; current tile position
Dim ItemDead(MaxItems)

; Signs
;----------

Dim SignX(5),SignY(5),SignText$(5)
Global SignActive;	0-none,1-on stinky,2-on loof (only for level-signs, i.e. #11-#15)

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

Global SFXTeleport
Global SFXDeath, SFXLevelLose, SFXButton, SFXTimerButton, SFXLevelWin, SFXHmmm, SFXTime
Global SFXStinkyCoin,SFXLoofCoin, SFXStinkyBonus, SFXLoofBonus, SFXLastCoin, SFXStinkyEnd, SFXLoofEnd
Global SFXIce,SFXStinkyConvey,SFXLoofConvey, SFXLava
Global SFXBoxPush, SFXBoxSplosh, SFXBoxExplo, SFXBreakAway, SFXBounce, SFXCoilyDeath, SFXBombExplo
Dim SFXTalk(20), SFXCount(10), SFXSlice(5)

Dim SFXSingle(10)	; 0-electro,1-cannonfire,2-cannonimpact
Dim SFXSingleVolume#(10)

Dim SFXMenuSound(10) ;0-menuselect,1-type



; ------------
; VARIOUS DATA
; ------------


; ------------
;     MENUS
; ------------

Global GameExit=False	; true if exit to windows
Global CurrentMenu
Global CurrentMenuOption
Global MenuBackground, MenuBackGround2; Entities for planes in background
Global MenuBackgroundtex, MenuBackGroundTex2; associated textures
Global Logo1, Logo2, Logotex1,Logotex2; the Stinky& Loof Logo
Global MenuSelector; the underlying bar for selected menu option
Global MenuSelectortex
Global MenuLevel=1
Global MenuStyle=-1
Global MenuBkg=-1
Global CursorKeyPressed, ShiftKeyPressed ; (in game, also used in end sequence)
Dim TitleChar(10)
Global Player2BDeleted
Global OptionsSoundLevel#=1.0
Global OptionsMusicLevel#=1.0
Dim DemoTexture(4)
Global IntroVoice=False

Global Scrolltext$
ScrollText$="                                                  "
If Version<10 
	ScrollText=ScrollText$+"Welcome to the free demo version of Wonderland.      Copyright @ 2002 Midnight Synergy.  Register your copy today at www.midnightsynergy.com              "
Else
	ScrollText=ScrollText$+"Welcome to Wonderland.     Copyright @ 2002 Midnight Synergy.  "
	ScrollText=ScrollText$+"    For more information on this and other games visit www.midnightsynergy.com          "
EndIf
ScrollText=ScrollText$+"   Credits:            Design, Code and Audiovisuals - Patrick Maidorn.       "
ScrollText=ScrollText$+"Music - Jonne 'Purple Motion' Valtonen.       Additional 3D Modelling - Danny Hawrysio.                          "
ScrollText=ScrollText$+"Thanks and Hello to Joerg Henseler, Scott Host, as well as Mark Sibly and the rest of the Blitz 3D community.                          " 
ScrollText=ScrollText$+"Special Thanks to my family, especially to 'Q' and Claire (the original Stinky and Loof!).       "
ScrollText=ScrollText$+"                                   "
ScrollText=ScrollText$+"    Are you still reading this?        How about some "
ScrollText=ScrollText$+"'fascinating' technical details?    Wonderland was written entirely in Blitz 3D (www.blitzbasic.co.nz), and as of v1.00 consisted of over 12,000 lines of code. "
ScrollText=ScrollText$+"3D Characters were modelled and animated in Milkshape 3D (www.swissquake.ch/chumbalum-soft). "
ScrollText=ScrollText$+"2D Textures and Graphics were created/modified in ArtGem (www.rlvision.com)."
ScrollText=ScrollText$+"                                                     "
ScrollText=ScrollText$+"Support Shareware - Register a Game today!"
ScrollText=ScrollText$+"                                                     "
ScrollText=ScrollText$+"                                                     "

; --------------
;     PLAYERS
; --------------
Dim PlayerName$(10)
Dim PlayerLevels(10)
Dim PlayerScore(10)	
Global PlayerNameN	; how many players currently on disk (max 10)
Global CurrentPlayerName$
Global CurrentPlayerScore
Dim CurrentPlayerLevelScore(500) ; 0 means not completed, otherwise score of that level
					; 0-9 world 1, 10-19 world 2, etc. 


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
Global CurrentLevel			; 	Level currently selected

Global CurrentWorldLevelN	; 	How many levels in this world?
Dim CurrentWorldLevelEntity(13)	; the squares for each level in the selector screen
								; up to 10 levels, + back/forward. #13 is always cursor
Global WorldLevelTexture;	a single texture containing the four symbols (no,yes,back,forward)
Global WorldLevelTexture2;	the cursor

Dim WorldLevelName$(500)		; filenames for each level
Dim WorldName$(10,10)				; descriptorname for each world
Dim CurrentWorldLevelName$(12,3); descriptorname for each level in world (3 lines)
								;	(loaded at start of levelselectorscreen)

Restore WorldLevelNames
For j=0 To 2 ; languages
	For i=0 To 9
		Read WorldName$(i,j)
	Next
Next

For i=0 To 499
	Read WorldLevelName$(i)
Next
If Version<10
	For i=10 To 14
		WorldLevelName$(i)="demo"+Str$(i-9)+".lev"
	Next 
	For i=15 To 19
		WorldLevelName$(i)=""
	Next
	If IsExtraDemoLevelActive=1
		WorldLevelName$(15)="resdb1.dat"
		WorldLevelName$(16)="resdb2.dat"
	EndIf

EndIf

Global PlayGroundLevel=400	;first level of The PlayGround


Dim DemoEndText$(2,100)
Restore DemoEndTexts
For j=0 To 37
	Read DemoEndText$(0,j)
Next
For j=0 To 57
	Read DemoEndText$(1,j)
Next

Dim DemoEndTextTimer(2)
DemoEndTextTimer(0)=0
DemoEndTextTimer(1)=0

.DemoEndTexts
Data ""
Data "* Includes Free Lifetime"
Data "  Product Updates."
Data ""
Data "* New graphics and music."
Data ""
Data "* 85 Levels in 10 Worlds,"
Data "  featuring mind-bending "
Data "  puzzles and exciting"
Data "  action challenges."
Data ""
Data "* Order Now and also get"
Data "  Five Free Bonus Levels!"
Data ""
Data "* New Gameplay Elements:"
Data "  Ice, Lava, Timers, "
Data "  ZBots, and much more."
Data ""
Data "* NEW: includes free "
Data "  access to our online "
Data "  community. Meet players, "
Data "  share hints, enter"
Data "  competitions, and much, "
Data "  much more."
Data ""
Data "* Safe, secure and fast "
Data "  online ordering... you "
Data "  could be playing the"
Data "  full game in 10 minutes."
Data ""
Data "* Try the full game risk-"
Data "  free with our money-"
Data "  back-guarantee."
Data ""
Data "* Find out why thousands "
Data "  of players from across "
Data "  the globe have made"
Data "  Wonderland their home."



Data "Smooth 3D graphics,"
Data "pleasant arcade music, and"
Data "touching characters."
Data "         Cnet Download.com"
Data ""
Data "I would recommend this "
Data "game to anyone looking "
Data "for an entertaining way "
Data "to challenge their minds."
Data "                  FileCart"
Data ""
Data "A breath of fresh air... "
Data "it will burn dozens of"
Data "hours of your time."
Data "                 InfoGames"
Data ""
Data "Deceptively simple... "
Data "lots of fun for both "
Data "children and adults."
Data "     Free Downloads Center"
Data ""
Data "Lovingly executed... a"
Data "game you'll want to come"
Data "back to again and again."
Data "              GamersEurope"
Data ""
Data "Remember when people made "
Data "simple and fun games with "
Data "tons of levels? "
Data "Midnight Synergy does."
Data "                PvP Online"
Data ""
Data "Good looking and good "
Data "fun... Wonderland is a"
Data "game that all the family "
Data "can join in and have a go."
Data "          Quandary Reviews"
Data ""
Data "A game that will exercise "
Data "your mind and let you "
Data "enjoy it too... definitely "
Data "a worthwhile game to get."
Data "                 BoltGames"
Data ""
Data "The puzzle designs "
Data "themselves are always "
Data "interesting and well "
Data "thought out."
Data "                IndieGamer"
Data ""
Data "Delightful... If you were "
Data "a puzzle addict and if you"
Data "enjoy playing classic "
Data "arcade games, you should "
Data "not be disappointed with "
Data "this game."
Data "           Magical Kingdom"
Data ""




.WorldLevelNames
; Worlds
Data "TRAINING GROUNDS","THE WATER PALACE","WINTER IN WONDERLAND","THE HALLS OF UNDERMOUNTAIN","TOP OF THE WORLD","DESERT CITIES","SPACED OUT","MIXED UP STINKY & LOOF","HEART OF THE VOLCANO","THE JADE TEMPLE"
Data "UEBUNGSGELAENDE","DER WASSERPALAST","WINTER IM WONDERLAND","DIE SAELE IM GEBIRGE","DACH DER WELT","WUESTENSTAEDTE","IM WELTRAUM","STINKY & LOOF","IM HERZEN DES VULKANS","DER JADE-TEMPEL"
Data "L'ENTRAINEMENT","LE PALAIS D'EAU","L'HIVER A WONDERLAND","LES GALERIES DE LA CAVERNE","LE SOMMET DU MONDE","LES VILLES DU DESERT","PERDU DANS L'ESPACE","STINKY & LOOF CONFONDUS","AU COEUR DU VOLCAN","LE TEMPLE DE JADE"

; Levels	
; Training Grounds
Data "tut1.lev","tut2.lev","tut3.lev","tut4.lev","tut5.lev","","","","",""
; The Water Palace
Data "castleb.lev","fishy.lev","coily.lev","friends.lev","telebroken.lev","airlock.lev","loofrescue.lev","bombintro.lev","",""
; Winter in Wonderland
Data "iceintro.lev","centreice.lev","fishbarrel.lev","heretheycome.lev","collide.lev","soclose.lev","ricochet.lev","spiral.lev","steelintro.lev","fireice.lev"
; The Halls of Undermountain
Data "trialfire.lev","guarcent.lev","claus1.lev","hounds.lev","blastthru.lev","crossfire.lev","moneymaze.lev","barrel.lev","corridor.lev",""
; Top of the World
Data "robotintro.lev","slip.lev","towerhop.lev","teamwork.lev","robowater.lev","","","","",""
; Desert Cities
Data "temple.lev","longway.lev","separate.lev","jones.lev","river.lev","greedy.lev","sort.lev","whoops.lev","moats.lev","prisoners.lev"
; Spaced Out
Data "tictac.lev","robotcity.lev","tight.lev","pacman.lev","timerintro.lev","timegates.lev","pushshove.lev","scouge.lev","",""
; Mixed Up Stinky & Loof
Data "network.lev","dizzy.lev","tensecs.lev","slipslide.lev","boxfactory.lev","","","","",""
; Heart of the Volcano
Data "cavein.lev","firewall.lev","bombsalot.lev","armoury.lev","bombwalk.lev","onecoin.lev","readyrun.lev","shortsweet.lev","stuck.lev","teletrap.lev"
; The Jade Temple
Data "oneshot.lev","threebox.lev","evasive.lev","return.lev","gofirst.lev","teleconfound.lev","guarbut.lev","walk.lev","capital.lev","lastone.lev"
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
; The PlayGround - World #40
Data "play1.lev","play2.lev","play3.lev","play4.lev","play5.lev","bonus1.lev","bonus2.lev","bonus3.lev","bonus4.lev","bonus5.lev"
Data "theend.lev","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""
Data "","","","","","","","","",""

; LevelNames and Signs - Translations
; German
; World 1
.GermanText
Data "Erste Schritte"
Data "Wilkommen im Wonderland!#Steuer 'Stinky' mit den Pfeiltasten#und sammle alle Regenbogen Taler auf.# "
Data "Kisten verschiebst du,#indem du gegen sie laeufst.#Schiebe die Kiste zur Seite#um deinen Weg fortzusetzen.# "
Data "Verborgene Schatzkammern,#so wie diese hier,#bringen dir oft Bonus Punkte.# # "
Data "Eine Kiste kann nur bewegt werden,#wenn der Platz hinter ihr frei ist.# #Es kann auch nur jeweils#eine einzige Kiste bewegt werden."
Data "Sobald du alle Regenbogen Taler eingesammelt#hast, bewege dich zum Regenbogen#Tor, umd das Level zu beenden.# # "

Data "Vorsicht: Falle"
Data "Sammle acht Regenbogen Taler ein, und#kehre hierhin zurueck sobald du alle#Taler gefunden hast.# # "
Data "Achte auf die Schurken mit den Kanonen.#Plane deine Schritte vorsichtig.# # # "
Data "Ein Laufband ist gut und schnell,#aber es transportiert dich nur#in eine Richtung.# # "
Data "Achtung!#Hochspannung!# # # "
Data "Verschiebe die Kisten, um die Bonus Punkte#hinter diesen zu bekommen. Zur Erinnerung:#du kannst auch ein Level beenden,#wenn du nicht alle Bonuspunkte#eingesammelt hast."

Data "Schalter und Stege"
Data "Gehe auf einen Schalter, um die #gleichfarbige Schranke zu deaktivieren.# # # "
Data "Runde Schalten bleiben nur gedrueckt,#solange etwas auf ihnen steht.# # # "
Data "Benutze diesen Schalter,#umd das Laufband auszuschalten. # # # "
Data "Kisten schwimmen auf dem Wasser.#Schiebe eine Kiste in das Wasser#um einen Steg zu bauen.# # "
Data "Gut gemacht!#Jetzt benutze drei Kisten,#um einen langen Steg zu bauen.#('Esc' wenn du nicht mehr weiter kommst)# "

Data "Schau Dich Um"
Data "Einige Levels sind zu gross, um alles auf einmal#zu sehen. Halte die 'Shift'-Taste fest#und benutze gleichzeitig die Pfeiltasten,#um das ganze Level zu erkunden.#Plane vor!"
Data "","","",""

Data "Darf ich vorstellen: 'Loof'"
Data "In einigen Levels spielst du sowohl#mit 'Stinky' als auch mit 'Loof'.#Benutze die 'Enter' Taste um zwischen#beiden zu wechseln.# "
Data "Lasse 'Stinky' und 'Loof' bei einigen#Aufgaben zusammen arbeiten,#aber achte darauf, dass#beide den Ausgang erreichen.# "
Data "","",""

; The following are World 2 Demo Levels:

Data "Nicht ganz geheuer!"
Data "","","","",""

Data "Achtung: 'Coily'!"
Data "Mach schnell# # # # "
Data "Warnung!!!#Passe auf 'Coily' auf!# # # "
Data "Hmmm...# # # # "
Data ""
Data ""

Data "Erst bist du hier, dann bist du dort"
Data "Teleporter funktionieren wie schnelle Wege.#Der rote Teleporter transportiert dich#zu dem anderen roten Teleporter,#zu deiner Rechten# "
Data "Der gelbe Teleporter wird nicht funktionieren,#da sein Ausgang von einer Kiste#blockiert ist. Du musst erst einen Weg#finden, um diese Kiste wegzubewegen.# "
Data "Hmmm...# # # # "
Data "Hilfe beim Stegbau?#Eine Kiste ist hinter dem blauem Teleporter.#Finde sie und bringe sie hir hin.# # "
Data ""

Data "Eins, Zwei, Kaboom!"
Data "'Kaboom!' gehoeren zu den Hitzkoepfen Wonderlands.#Komme ihnen nicht zu nahe!# # # "
Data "Kaboom!s lassen sich nicht gerne einfangen,#und explodieren wenn sie eingeschlossen sind.# # # "
Data "Kaboom!s lassen sich nicht gerne einfangen,#und explodieren wenn sie eingeschlossen sind.# # # "
Data ""
Data ""

Data "Loof kommt zur Rettung!
Data "Hmmm...# # # # ","","","",""

Data "So nahe!#(und doch so fern)"
Data "Hmmm...# # # # ","Hurra!!!# # # # ","","",""

Data "Pfad des Feuers"
Data "Hmmm...# # # # ","Hurra!!!# # # # ","","",""



; Now World 2 Normal

Data "Holzstege"
Data "Vorsicht: Holzstege tragen dich nur einmal#und sind fuer Kisten nicht stabil genug.# # # "
Data "Zur Erinnerung:#Benutze 'Shift' und die Pfeiltasten,#um einen Level zu erkunden.# # "
Data "Wenn du dieses mal nicht alle Bonus#Punkte einsammeln konntest, kannst du#das Level jederzeit wiederholen.# # "
Data "Baue dir einen Fluchtweg,#bevor du die Bruecke nach Sueden passierst.# # # "
Data ""

Data "Nicht ganz geheuer!"
Data "","","","",""

Data "Achtung: 'Coily'!"
Data "Mach schnell# # # # "
Data "Warnung!!!#Passe auf 'Coily' auf!# # # "
Data "Hmmm...# # # # "
Data ""
Data ""

Data "Mit ein wenig Hilfe meiner Freunde"
Data "","","","",""

Data "Erst bist du hier, dann bist du dort"
Data "Teleporter funktionieren wie schnelle Wege.#Der rote Teleporter transportiert dich#zu dem anderen roten Teleporter,#zu deiner Rechten# "
Data "Der gelbe Teleporter wird nicht funktionieren,#da sein Ausgang von einer Kiste#blockiert ist. Du musst erst einen Weg#finden, um diese Kiste wegzubewegen.# "
Data "Hmmm...# # # # "
Data "Hilfe beim Stegbau?#Eine Kiste ist hinter dem blauem Teleporter.#Finde sie und bringe sie hir hin.# # "
Data ""

Data "Eingesperrt"
Data "","","","",""

Data "Loof kommt zur Rettung!
Data "Hmmm...# # # # ","","","",""

Data "Eins, Zwei, Kaboom!"
Data "'Kaboom!' gehoeren zu den Hitzkoepfen Wonderlands.#Komme ihnen nicht zu nahe!# # # "
Data "Kaboom!s lassen sich nicht gerne einfangen,#und explodieren wenn sie eingeschlossen sind.# # # "
Data "Kaboom!s lassen sich nicht gerne einfangen,#und explodieren wenn sie eingeschlossen sind.# # # "
Data ""
Data ""

Data "","","","","",""
Data "","","","","",""

Data "RutschPartie"
Data "Passe auf das Eis auf...# # # # "
Data "Einige Eisfelder haben natuerliche Kurven...#folgen ihren Pfaden und du siehst,#wohin sie dich tragen werden.# # "
Data "Achtung:# rutsche nicht in etwas Gefaehrliches#wie z.B. Wasser!# # "
Data "",""

Data "Eiszentrum"
Data "","","","",""

Data "Coily im Fadenkreuz"
Data "","","","",""

Data "Hier kommen sie..."
Data "","","","",""

Data "Kollisionskurs"
Data "Hinweis:#Kisten in Bewegung#vernichten sich gegenseitig# # ","","","",""

Data "So nahe!#(und doch so fern)"
Data "","","","",""

Data "Abpraller"
Data "","","","",""

Data "Die Spirale des Schicksals"
Data "","","","",""

Data "Holz und Stahl"
Data "Stahlkisten bewegst du genauso wie Holzkisten.# # # # "
Data "Stahlkisten blockieren Feuerkugeln wirkungsvoll!# # # # "
Data "Bei einer Kollision#ist Holz nicht so stark wie Stahl.# # # "
Data "Stahlkisten sind wunderbar,#nur eines koennen sie nicht...#auf dem Wasser schwimmen!# # "
Data "Hinweis:#Du brauchst sowohl 'Stinky' als auch 'Loof'#um die Bonus Punkte zu bekommen!# # "

Data "Feuer und Eis"
Data "","","","",""

Data "Pfad des Feuers"
Data "","","","",""

Data "Das bewachte Zentrum"
Data "","","","",""

Data "Platzangst"
Data "","","","",""

Data "Vorsicht: Kaboom!"
Data "","","","",""

Data "Explosionskessel"
Data "","","","",""

Data "Kreuzfeuer"
Data "","","","",""

Data "Der Irrgarten"
Data "Vorsichtig...#Glaube nicht, du bist alleine!# # # "
Data "Durchkommen war leicht...#nur wie viele Bonus Punkte#hast du bekommen?# # "
Data "","",""

Data "Gewehrlauf"
Data "","","","",""

Data "Der Korridor"
Data "Geduld zahlt sich aus.# # # # ","","","",""

Data "","","","","",""

Data "Z-Bot"
Data "Z-Bots folgen geradlinig ihren Wegen,#bis sie auf ein Hindernis treffen.#Lass dich nicht von ihnen erwischen!# # "
Data "Hinweis:#Z-Bots sind in der Lage, Schalter zu aktivieren.#Benutze zwei Kisten,# um diesen Z-Bot am richtigen Platz festzuhalten.# # "
Data "","",""

Data "Die zwei Tuerme"
Data "Z-Bots vertragen kein Wasser.#Benutze diese Kiste um Z-Bots#in das Wasser zu stupsen.# # "
Data "","","",""

Data "Turmsprung"
Data "Hinweis:#Eine der Bodenplatten ist nicht wie die anderen!# # # "
Data "","","",""

Data "Teamwork"
Data "","","","",""

Data "Bitte nicht Nass machen!"
Data "Bonuszeit!#Sammle viele Punkte,#indem du Z-Bots in den Wassergraben schickst!# # "
Data "","","",""

Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""

Data "Der Tempel"
Data "Kannst du die Bonuspunkte unten erreichen?# # # # "
Data "","","",""

Data "Der lange Weg nach Hause"
Data "","","","",""

Data "Loof... Wo bist du?"
Data "","","","",""

Data "Jaeger der verlorenen Muenzen"
Data "","","","",""

Data "Der Fluss"
Data "","","","",""

Data "Wie gierig bist du?"
Data "","","","",""

Data "Symmetrie"
Data "","","","",""

Data "Was dieser Schalter alles anstellt..."
Data "","","","",""

Data "Der Gipfel"
Data "","","","",""

Data "Gefaengnisturm"
Data "","","","",""

Data "Tic Tac Toe"
Data "","","","",""

Data "Z-Bot Stadt"
Data "","","","",""

Data "Nicht viel Platz hier!"
Data "","","","",""

Data "Das sieht sehr vertraut aus?!?
Data "","","","",""

Data "Zeitliche Raetsel"
Data "Sternfoermige Schalter sind mit Timer verbunden# # # # "
Data "","","",""

Data "Mach schnell!!!"
Data "Vorsichtig!#Alle Tore sind mit Timer verbunden!# # # "
Data "Noch drei weitere Tore...# # # # "
Data "Zwei noch...# # # # "
Data "Nur noch dieses!!!# # # # "
Data ""

Data "Schiebung!"
Data "","","","",""

Data "Dieser Schuft!"
Data "","","","",""

Data "","","","","",""
Data "","","","","",""

Data "Netzwerktheorie"
Data "","","","",""

Data "Mir wird schwindelig!"
Data "","","","",""

Data "Nur Zehn Sekunden???"
Data "","","","",""

Data "Rrrrrrutsch!"
Data "","","","",""

Data "Die Verrueckte Kisten-Fabrik"
Data "Achtung!#Konstruktions Zone!# # # ","","","",""

Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""

Data "Nach dem Einbrechen"
Data "Passe auf die Lava auf.#Sie wird alle Holzkisten vernichten.#Nur Stahl kann ihr wiederstehen.# # "
Data "","","",""

Data "Breitseite!"
Data "","","","",""

Data "Jede Menge Bomben"
Data "","","","",""

Data "Die Waffenkammer"
Data "","","","",""

Data "Zuhause mit den Kaboom!s"
Data "","","","",""

Data "Nur ein Taler!"
Data "Ich glaube nicht,#dass du schnell genug bist#denTaler zu bekommen!# # "
Data "","","",""

Data "Auf die Plaetze, fertig und los!"
Data "","","","",""

Data "Klein... aber nicht so fein"
Data "Und nun, der nicht ganz so feine Teil...# # # # ","","","",""

Data "Wie komme ich hier raus?"
Data "","","","",""

Data "Gefangen im Teleporter"
Data "","","","",""

Data "Du hast nur eine Chance"
Data "","","","",""

Data "Versteckte Schaetze"
Data "","","","",""

Data "Nix wie weg!"
Data "","","","",""

Data "Hin und wieder zurueck"
Data "Vorsicht!# # # # ","","","",""

Data "Geh du zuerst"
Data "","","","",""

Data "Verwechselte Teleporter"
Data "","","","",""

Data "Gesicherte Schalter"
Data "","","","",""

Data "Ein Spaziergang"
Data "","","","",""

Data "Was fuer eine Aussicht!"
Data "","","","",""

Data "Einen hamm wir noch!"
Data "Eine Karte koennte helfen.# # # # ","","","",""

Data "Puzzle 101"
Data "Wie viele Bonus Punkte#kannst du bekommen?# # # "
Data "","","",""

Data "Skate Wonderland"
Data "","","","",""

Data "Ein Tag im Zoo"
Data "Coilys!# # # # "
Data "Bitte nicht fuettern!# # # # "
Data "Willkommen im Wonderland Zoo# # # # "
Data "Z-Bots... Achtung elektrisch!# # # # "
Data "Kaboom!s# # # # "

Data "Hindernislauf"
Data "Wie schnell bist du?# # # # ","","","",""

Data "Finde den Weg"
Data "Es sind 12 Taler in diesem Irrgarten verborgen.#Wirst du alle finden?# # # ","","","",""

Data "Bonus 1","","","","",""
Data "Bonus 2","","","","",""
Data "Bonus 3","","","","",""
Data "Bonus 4","","","","",""
Data "Bonus 5","","","","",""

Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""

; French
; World 1
.FrenchText
Data "Les Premiers Pas"
Data "Bienvenue a Wonderland!#Controle 'Stinky' avec les fleches,#et ramasse tous les sous Arc-en-ciel#pour finir un niveau.# "
Data "Les Boites peuvent etre deplacees#en marchant sur elles.#pousse cette Boite sur le cote#pour liberer le passage.# "
Data "Souvent, des cachettes au tresor,#comme celle-ci,#revele d'autres Points Bonus.# # "
Data "Tu peux bouger une Boite seulement# s'il n'y a rien derriere.# #Aussi, tu ne peux bouger#qu'une Boite a la fois."
Data "Une fois tous les sous Arc-en-ciel ramasses,#franchis la Grille Arc-en-ciel pour#terminer le niveau.# # "

Data "Pieges et Tribulations
Data "Il y a huit Sous Arc-en-ciel a ramasser ici.#Reviens a cette Grille une fois tous les#Sous ramasses. Le nombre de#Sous Arc-en-ciel restants est toujours#affiche dans le coin superieur gauche de l'ecran."
Data "Attention a Scouge, le Canon!#Planifie tes etapes avec soin!# # # " 
Data "Les tapis roulants sont amusants,#et rapides, mais ils sont a sens unique.# # # "
Data "Danger!#Haut Voltage!# # # "
Data "Arrange ces Boites pour obtenir les Points Bonus!# #Souviens-toi:#Tu peux quand meme terminer un niveau sans avoir#ramasse tous les Points Bonus."

Data "Boutons et Ponts"
Data "Marcher sur un bouton eteindra#les rayons de la meme couleur.# # # "
Data "Les boutons ronds se rallumeront si il n'y a#rien dessus. Trouve un truc#pour qu'ils restent eteints.# # "
Data "Ce bouton sert a#arreter le tapis roulant.# # # "
Data "Les Boites flottent sur l'eau.#Pousse la Boite dans l'eau#pour faire un pont.# # "
Data "Bravo!#Bon, prends trois Boites pour#faire un pont plus long.# #(tape Esc si tu restes coince)"

Data "Comtemple le Panorama"
Data "Certains niveaux sont tres grands et debordent#l'ecran. Tiens la touche 'Maj.', puis utilise#les fleches pour explorer ce niveau.# #Planifie! "
Data "","","",""

Data "Et le voici : Loof"
Data "Dans certains niveaux tu joueras avec#les deux: 'Stinky' et 'Loof'.# #Alterne entre les deux avec la touche 'Entree'.# "
Data "Utilises 'Stinky' et 'Loof' en#cooperation dans les casse-tetes, mais#les DEUX doivent parvenir a la sortie!# # "
Data "","",""

; The following are World 2 Demo Levels:

Data "Mordez, mes P'tits Vlimeux!"
Data "","","","",""

Data "Coily lache lousse!"
Data "Le temps n'est pas avec toi.#Tu as seulement trois minutes pour#achever ce niveau!# # "
Data "Attention!!!#Mefie-toi de 'Coily'!!!# # # "
Data "Ils sont trop nombreux!# # # # "
Data ""
Data ""

Data "La tu y es, puis la, tu y es plus..."
Data "Les Teleporteurs sont comme des#passages instantanes.#Ce Teleporteur rouge te menera #a cet autre Teleporteur rouge a ta droite.# "
Data "Ce Teleporteur jaune a ta droite#ne fonctionnera pas, parce que sa#sortie est bloquee par une Boite.# #Trouve d'abord une facon d'enlever cette Boite! "
Data "Tu veux attraper ces Points Bonus?# #En entrant dans un Teleporteur, ton #orientation ne change pas. Alors comment#passeras-tu de l'autre cote du Teleporteur vert? "
Data "Tu aurais besoin d'un pont?# #Il y a une Boite a l'autre extremite du#Teleporteur bleu. Trouve-la et ramene-la!# "
Data ""

Data "Deux, Un, Kaboom!"
Data "Et voici 'Kaboom!', #a Wonderland, y a pas plus 'tete brulee'#que ces bou-boules.# #Ne les frole pas! "
Data "Les Kaboom!s n'aimes pas etre a l'etroit,#et exploseront s'ils sont mis en Boite.# # # "
Data "Quand un Kaboom est coince, #il eclate et detruit tout ce qu'il y a autour.# #Utilise ce savoir pour te frayer un chemin.# "
Data ""
Data ""

Data "Loof a la rescousse!"
Data "Hmmm...# # # # ","","","",""

Data "Si Pres!#(et pourtant si loin)"
Data "Hmmm...# # # # ","Yippi!!!# # # # ","","",""

Data "L'Epreuve du Feu"
Data "Hmmm...# # # # ","Yippi!!!# # # # ","","",""


; Now World 2 Normal

Data "Le Traversee des Douves"
Data "Mise en garde!#Les plaques de Bois ne te supporteront qu'une#seule fois, et elles sont trop fragiles#pour supporter les Boites.# "
Data "Souviens-toi! 'Maj.' et les Fleches#pour survoler le niveau.# # # "
Data "Si tu ne peux ramasser#tous les Points Bonus cette fois-ci,#N'oublie pas que tu peux rejouer tous les niveaux#a ta guise pour tenter d'obtenir le Score Parfait.# "
Data "Construis une Voie de Sortie#avant de traverser le Pont-du-Sud!# # # "
Data ""

Data "Mordez, mes P'tits Vlimeux!"
Data "","","","",""

Data "Coily lache lousse!"
Data "Le temps n'est pas avec toi.#Tu as seulement trois minutes pour#achever ce niveau!# # "
Data "Attention!!!#Mefie-toi de 'Coily'!!!# # # "
Data "Ils sont trop nombreux!# # # # "
Data ""
Data ""

Data "'With a little Help from my Friends'"
Data "","","","",""

Data "La tu y es, puis la, tu y es plus..."
Data "Les Teleporteurs sont comme des#passages instantanes.#Ce Teleporteur rouge te menera #a cet autre Teleporteur rouge a ta droite.# "
Data "Ce Teleporteur jaune a ta droite#ne fonctionnera pas, parce que sa#sortie est bloquee par une Boite.# #Trouve d'abord une facon d'enlever cette Boite! "
Data "Tu veux attraper ces Points Bonus?# #En entrant dans un Teleporteur, ton #orientation ne change pas. Alors comment#passeras-tu de l'autre cote du Teleporteur vert? "
Data "Tu aurais besoin d'un pont?# #Il y a une Boite a l'autre extremite du#Teleporteur bleu. Trouve-la et ramene-la!# "
Data ""

Data "Cadenasse"
Data "","","","",""

Data "Loof a la rescousse!"
Data "Hmmm...# # # # ","","","",""

Data "Deux, Un, Kaboom!"
Data "Et voici 'Kaboom!', #a Wonderland, y a pas plus 'tete brulee'#que ces bou-boules.# #Ne les frole pas! "
Data "Les Kaboom!s n'aimes pas etre a l'etroit,#et exploseront s'ils sont mis en Boite.# # # "
Data "Quand un Kaboom est coince, #il eclate et detruit tout ce qu'il y a autour.# #Utilise ce savoir pour te frayer un chemin.# "
Data ""
Data ""

Data "","","","","",""
Data "","","","","",""

Data "Les Glissades"
Data "Attention aux surfaces glacees...# # # # "
Data "Quelques plaques de glace ont des#courbes naturelles...#suis leurs traces pour#voir ou ca te menera.# "
Data "Prends Garde:# #Ne te mets pas les pieds dans les plats,#...les plats d'eau par exemple.# "
Data "",""

Data "Mise au Jeu"
Data "","","","",""

Data "La Peche au P'Tit Poisson"
Data "","","","",""

Data "Les voici!"
Data "","","","",""

Data "Ligne d'Impact"
Data "Indice:#Les Boites en mouvement#se briseront l'une contre l'autre.# # "
Data "","","",""

Data "Si Pres!#(et pourtant si loin)"
Data "","","","",""

Data "Ricochets"
Data "","","","",""

Data "La Spirale du Destin"
Data "","","","",""

Data "Bois et Acier"
Data "Les Boites d'acier se comportent exactement#comme les Boites de Bois.# # # "
Data "Boites d'Acier bloquent#efficacement les boules de feu!# # # "
Data "Lors d'un impact, #le bois ne vaut pas l'acier.# # # "
Data "Les Boites d'acier sont geniales, sauf pour#une chose... elles ne...# # # ...flottent pas!"
Data "Indice:# #Stinky ET Loof te seront indispensables#pour atteindre ces Points Bonus!# "

Data "Le Feu et l'Eau"
Data "","","","",""

Data "L'Epreuve du Feu"
Data "","","","",""

Data "Centre sous Surveillance"
Data "","","","",""

Data "Claustrophobie"
Data "","","","",""

Data "Lachez les Chiens!"
Data "","","","",""

Data "Defonce-toi!"
Data "","","","",""

Data "Feux Croises!"
Data "","","","",""

Data "Le Labyrinthe de Dollars"
Data "Prudence... Je ne crois pas que tu sois Seul!# # # # "
Data "Le Parcours etait Facile...# #... mais combien as-tu recolte de Points Bonus?# # "
Data "","",""

Data "Le Barillet du Revolver"
Data "","","","",""

Data "Le Corridor"
Data "Tout vient a Point ... a qui sait Attendre... # # # "
Data "","","",""

Data "","","","","",""

Data "Moi, Zi-Bot"
Data "Les Zi-Bots vont en ligne droite jusqu'a ce#qu'ils frappent un obstacle et tournent#a gauche ou a droite.#Ne te fais pas Attraper!# "
Data "Truc:#Un Zi-Bot peut aussi declencher les Boutons.#Utilise les deux Boites pour leurrer#ce Zi-Bot au bon endroit!# "
Data "","",""

Data "Les Deux Tours"
Data "Zi-Bots Reagissent Tres Mal a l'Eau.# #Utilise cette Boite pour 'Guider' les Zi-Bots#dans le bassin d'eau.# "
Data "","","",""

Data "Saute Mou-Tour"
Data "Indice: # #Une de ces Dalles n'est pas Comme les Autres!# # "
Data "","","",""

Data "L'Heure du Travail a la Chaine"
Data "","","","",""

Data "Ne pas Immerger!"
Data "L'Heure des Bonus!#Un gros Paquet de Points si#Tu Expedies les Zi-Bots a la Flotte!# # "
Data "","","",""

Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""

Data "Le Temple"
Data "Peux-tu atteindre les Points Bonus au bas?# # # # "
Data "","","",""

Data "La Route est Longue jusqu'a la Maison"
Data "","","","",""

Data "L'Angoisse de Separation"
Data "","","","",""

Data "Les Aventuriers des Sous Perdus"
Data "","","","",""

Data "Un Fleuve le Traverse"
Data "","","","",""

Data "Mon or, Viande a Chien!"
Data "","","","",""

Data "Symetries"
Data "","","","",""

Data "A Quoi ca sert ce Bouton..."
Data "","","","",""

Data "Le Pinacle"
Data "","","","",""

Data "Le Dongeon de la Tour"
Data "","","","",""

Data "Tic Tac Toe"
Data "","","","",""

Data "Zi-BotVille!"
Data "","","","",""

Data "Espace Vitale Reduit"
Data "","","","",""

Data "Y m'Semble que j'Connais Ca!?!"
Data "","","","",""

Data "Le Temps Passe Vite Quand on s'Amuse"
Data "Les Boutons en Forme d'Etoile#sont Relies aux Compteurs.# # # "
Data "","","",""

Data "Le temps est Relatif!"
Data "Attention!# #Chaque Grille est Reliee a un Compteur!# # "
Data "Encore Trois grilles...# # # # "
Data "Encore Deux...# # # # "
Data "Presque rendu!!!# # # # "
Data ""

Data "Quand Pousser devient Pelleter"
Data "","","","",""

Data "Ce Satane Scouge!"
Data "","","","",""

Data "","","","","",""
Data "","","","","",""

Data "La Theorie des Reseaux"
Data "","","","",""

Data "J'ai mal au Coeur!"
Data "","","","",""

Data "Dix Secondes???"
Data "","","","",""

Data "L'Etonnant 'Slip-And-Slide'"
Data "","","","",""

Data "L'Usine a Boites Vous Emballe"
Data "Mise en Garde!#Zone de Construction!# # # ","","","",""

Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""

Data "Apres l'Effondrement"
Data "Sois attentif a la Lave autour.# #Elle detruira toute Boite de Bois,#mais pas Celles en Acier...#...du Moins pas Tout de Suite..."
Data "","","",""

Data "Mur de Feu"
Data "","","","",""

Data "Bombes a Profusion"
Data "","","","",""

Data "L'Armurerie"
Data "","","","",""

Data "Le Guide Kaboom! :#Demolition pour Tous"
Data "","","","",""

Data "Seulement un Sou!"
Data "Me Semble que t'es pas Assez Vite#pour Attraper ce Sous!# # # "
Data "","","",""

Data "A vos marques, Pret, Partez!"
Data "","","","",""

Data "En deux Temps...mais Plus de trois Mouvements"
Data "Maintenant, pour la Partie Mouvementee...# # # # ","","","",""

Data "Coince!"
Data "","","","",""

Data "Piege dans le Teleporteur"
Data "","","","",""

Data "Une Seule Chance"
Data "","","","",""

Data "Tresors Caches"
Data "","","","",""

Data "Les Contre-Mesures"
Data "","","","",""

Data "Le Voyage du Retour"
Data "Prepare toi a courir!# # # # ","","","",""

Data "Toi en Premier!"
Data "","","","",""

Data "Teleporteurs Trompeurs"
Data "","","","",""

Data "Les Boutons Proteges"
Data "","","","",""

Data "Allons Prendre une Marche"
Data "","","","",""

Data "Quelle Vue!"
Data "","","","",""

Data "Un p'tit pour la route..."
Data "Une Carte Peut-etre?# # # # ","","","",""

Data "Casse-Tete 101"
Data "Combien de Points Peux-tu Attraper?# # # "
Data "","","",""

Data "Le Patin a Wonderland"
Data "","","","",""

Data "Une Journee au Zoo"
Data "Sacripants de Coilys!# # # # "
Data "S.V.P. Ne pas Nourrir les Poissons!# # # # "
Data "Bienvenue au Zoo Royal de Wonderland# #Aidez-nous a Garder#ce Site Propre! "
Data "Zi-Bots...# #Attention a leurs Baisers Electriques!# # # "
Data "Voici Kaboom!# # # # "

Data "Course a Obstacles"
Data "Es-tu Rapide???# # # # ","","","",""

Data "Des Dales"
Data "Il y a Douze Sous#caches dans ce dedale.# #Tu peux les trouver tous?# # ","","","",""

Data "Bonus 1","","","","",""
Data "Bonus 2","","","","",""
Data "Bonus 3","","","","",""
Data "Bonus 4","","","","",""
Data "Bonus 5","","","","",""


Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""
Data "","","","","",""