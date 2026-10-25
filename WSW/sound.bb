Function LoadSounds()

	; Load Sound FX
	;SFXWalk=MyLoadSound("sound\walk.wav",True)
	
		
	SFXLevelLose=MyLoadSound("sound\death2.wav",False)
	;SFXLevelWin=MyLoadSound("sound\win.wav",True)
	SFXBreakAway=MyLoadSound("sound\breakaway.wav",False)
	SFXBoxPush=MyLoadSound("sound\boxpush.wav",False)
	SFXBoxSplosh=MyLoadSound("sound\splosh.wav",False)
	SFXBoxExplo=MyLoadSound("sound\boxexplo.wav",False)
	SFXTeleport=MyLoadSound("sound\teleport.wav",False)
	SFXButton=MyLoadSound("sound\button.wav",False)
	
	SFXBounce=MyLoadSound("sound\bounce.wav",False)
	SFXCoilyDeath=MyLoadSound("sound\coilydeath.wav",False)
	SFXBombExplo=MyLoadSound("sound\bombexplo.wav",False)
	
	SFXBoulderPush=MyLoadSound("sound\boulderpush.wav",False)
	SFXBoulderHit=MyLoadSound("sound\boulderhit.wav",False)
	
	If version>=10 SFXSingle(0)=MyLoadSound("sound\electro.wav",False)
	SFXSingle(1)=MyLoadSound("sound\fireshot.wav",False)
	SFXSingle(2)=MyLoadSound("sound\fireexplo.wav",False)
	If version>=10 SFXSingle(3)=MyLoadSound("sound\saucershot.wav",False)
	If version>=10 SFXSingle(4)=MyLoadSound("sound\saucerfly.wav",False)
	SFXSingle(5)=MyLoadSound("sound\spikeup.wav",False)
	SFXSingle(6)=MyLoadSound("sound\spikedown.wav",False)
	SFXSingle(7)=MyLoadSound("sound\reflector.wav",False)
	SFXSingle(8)=MyLoadSound("sound\chomper1.wav",False)
	SFXSingle(9)=MyLoadSound("sound\chomper1.wav",False)
	SFXSingle(10)=MyLoadSound("sound\chomper1.wav",False)
	SFXSingle(11)=MyLoadSound("sound\chomper1.wav",False)
	If version>=10 SFXSingle(12)=MyLoadSound("newsounds\prism1.wav",False)
	If version>=10 SFXSingle(13)=MyLoadSound("newsounds\boxgen1.wav",False)
	If version>=10 SFXSingle(14)=MyLoadSound("newsounds\mother1.wav",False)
	If version>=10 SFXSingle(15)=MyLoadSound("newsounds\spirit1.wav",False)
	
	
	UpdateWorld
	RenderWorld
	Text DisplayWidth/2,DisplayHeight*23/36,"LOADING",True,False

	Text DisplayWidth/2,DisplayHeight*2/3,"80% Complete",True,False
	Flip






	
	; stinky and loof voices
	
	SFXPlayerCoin(1)=MyLoadSound("sound\stinkycoin.wav",False)
	SFXPlayerBonus(1)=MyLoadSound("sound\stinkybonus.wav",False)
	SFXPlayerClock(1)=MyLoadSound("sound\stinkyclock.wav",False)
	SFXPlayerEnd(1)=MyLoadSound("sound\stinkyend.wav",False)
	SFXPlayerConvey(1)=MyLoadSound("sound\stinkyconvey.wav",False)
	
	SFXPlayerCoin(2)=MyLoadSound("sound\loofcoin.wav",False)
	SFXPlayerBonus(2)=MyLoadSound("sound\loofbonus.wav",False)
	SFXPlayerClock(2)=MyLoadSound("sound\loofclock.wav",False)
	SFXPlayerEnd(2)=MyLoadSound("sound\loofend.wav",False)
	SFXPlayerConvey(2)=MyLoadSound("sound\loofconvey.wav",False)
	
	If version>=10 
		SFXPlayerCoin(3)=MyLoadSound("newsounds\qookiecoin3.wav",False)
		SFXPlayerBonus(3)=MyLoadSound("newsounds\qookiecoin4.wav",False)
		SFXPlayerClock(3)=MyLoadSound("newsounds\qookietime1.wav",False)
		SFXPlayerEnd(3)=MyLoadSound("newsounds\qookieend1.wav",False)
		SFXPlayerConvey(3)=MyLoadSound("newsounds\qookiecoin5.wav",False)
		
		SFXPlayerCoin(4)=MyLoadSound("newsounds\peeguecoin3.wav",False)
		SFXPlayerBonus(4)=MyLoadSound("newsounds\peeguetime2.wav",False)
		SFXPlayerClock(4)=MyLoadSound("newsounds\peeguetime1.wav",False)
		SFXPlayerEnd(4)=MyLoadSound("newsounds\peegueend1.wav",False)
		SFXPlayerConvey(4)=MyLoadSound("newsounds\peegueconvey1.wav",False)
	EndIf


	
	If version>=10 SFXStinker=MyLoadSound("newsounds\stinker.wav",False)

	
	SFXStinkycoin=MyLoadSound("sound\stinkycoin.wav",False)
	SFXLoofcoin=MyLoadSound("sound\Loofcoin.wav",False)
	SFXStinkyBonus=MyLoadSound("sound\StinkyBonus.wav",False)
	SFXLoofBonus=MyLoadSound("sound\LoofBonus.wav",False)
	SFXLastCoin=MyLoadSound("sound\lastcoin.wav",False)
	SFXStinkyEnd=MyLoadSound("sound\StinkyEnd.wav",False)
	SFXLoofEnd=MyLoadSound("sound\LoofEnd.wav",False)
;	SFXStinkyIce=MyLoadSound("StinkyIce.wav",False)
;	SFXLoofIce=MyLoadSound("LoofIce.wav",False)
	SFXStinkyConvey=MyLoadSound("sound\StinkyConvey.wav",False)
	SFXLoofConvey=MyLoadSound("sound\LoofConvey.wav",False)
	SFXStinkyClock=MyLoadSound("sound\Stinkyclock.wav",False)
	SFXLoofClock=MyLoadSound("sound\LoofClock.wav",False)
	If version>=10 SFXSaucerDie=MyLoadSound("sound\SaucerDie.wav",False)
	
	UpdateWorld
	RenderWorld
	Text DisplayWidth/2,DisplayHeight*23/36,"LOADING",True,False

	Text DisplayWidth/2,DisplayHeight*2/3,"90% Complete",True,False
	Flip


	
	

	
	
	SFXHmmm=MyLoadSound("sound\slhmmm.wav",False)
	SFXTime=MyLoadSound("sound\sltime.wav",False)
	SFXDeath=MyLoadSound("newsounds\sldeathquiet.wav",False)
	
	If FileType("CustomVoices\start1.wav")=1
		SFXTalk(0)=MyLoadSound("CustomVoices\start1.wav",False)
		For i=1 To 4
			SFXTalk(i)=SFXTalk(0)
		Next
	Else
		SFXTalk(0)=MyLoadSound("sound\getready.wav",False)
		SFXTalk(1)=MyLoadSound("sound\herewego.wav",False)
		SFXTalk(2)=MyLoadSound("sound\cmonletsgo.wav",False)
		SFXTalk(3)=MyLoadSound("sound\letsdoit.wav",False)
		SFXTalk(4)=MyLoadSOund("newsounds\gitready.wav",False)
	EndIf
	
	
	
	
	If version>=10 
		SFXTalk(5)=MyLoadSound("sound\stinkyno.wav",False)
		SFXTalk(6)=MyLoadSound("newsounds\mynameisqookie2.wav",False)
		SFXTalk(7)=MyLoadSound("newsounds\peeguespell3.wav",False)
	EndIf
	
	
	
	SFXTalk(11)=MyLoadSOund("newsounds\charstinky.wav",False)
	SFXTalk(12)=MyLoadSOund("newsounds\charloof.wav",False)
	If version>=10 
		SFXTalk(13)=MyLoadSOund("newsounds\charqookie2.wav",False)
		SFXTalk(14)=MyLoadSOund("newsounds\charpeegue.wav",False)
	EndIf

	UpdateWorld
	RenderWorld
	Text DisplayWidth/2,DisplayHeight*23/36,"LOADING",True,False

	Text DisplayWidth/2,DisplayHeight*2/3,"100% Complete",True,False
	Flip

	SFXMenuSound(0)=MyLoadSound("sound\menumove.wav",False)
	SFXMenuSound(1)=MyLoadSound("sound\type.wav",False)
	For i=0 To 10
		SFXCount(i)=MyLoadSound("sound\sl"+Str$(i)+".wav",False)
	Next
	
	
	SFXTimerButton=MyLoadSound("sound\windup.wav",False)
	SFXIce=MyLoadSound("sound\ice.wav",False)
	For i=0 To 4
		SFXSlIce(i)=MyLoadSound("sound\slice"+Str$(i+1)+".wav",False)
	Next
	If version>=10 	SFXLava=MyLoadSound("sound\blub.wav",False)	
	
	
	SFXLightning(0)=MyLoadSound("sound\lightning1.wav",False)
	SFXLightning(1)=MyLoadSound("sound\lightning2.wav",False)
	SFXLightning(2)=MyLoadSound("sound\lightning3.wav",False)
	SFXLightning(3)=MyLoadSound("sound\lightning4.wav",False)



	
End Function


Function PlayMySound(bsound,bentity)
	If OptionsSoundLevel=0 Return
	If DoesSoundExist=False Return
	; if no distance specified
	If bentity=0 
		SoundVolume bsound,OptionsSoundLevel
		PlaySound(bsound)
		
		Return
	EndIf

	; calculate distance from sound
	TranslateEntity Camera,0,-10,9
	distance#=EntityDistance (Camera,bentity)
	TranslateEntity Camera,0,10,-9

	a#=3.0 ; minimum radius
	b#=9.0 ; maximum radius
	
	volume#=(b-distance)/(b-a)
	
	If OptionsSoundLevel*volume<=0 Then Return
	
	If volume>1 Then volume=1
	
	
	SoundVolume(bsound,OptionsSoundLevel*volume)
	PlaySound(bsound)	
	


	
End Function

Function PlaySingleSound(x,y,bType)

	; 0-electro
	; 1-cannon
	; 2-fireball
	; 3-saucershot
	; 4-saucerfly
	; 5-spikeup
	; 6-spikedown
	; 7-fireball reflected

	If OptionsSoundLevel=0 Return
	If DoesSoundExist=False Then Return
	
	If x=-999 
		SFXSingleVolume(btype)=1.0
		Return
	EndIf

	; just checks volume... at the end plays highest volume sound in UpdateGame

	bentity=CreatePivot()
	TranslateEntity bentity,x+.5,0,-y-0.5
	
	; calculate distance from sound
	TranslateEntity Camera,0,-10,9
	distance#=EntityDistance (Camera,bentity)
	TranslateEntity Camera,0,10,-9

	a#=3.0 ; minimum radius
	b#=9.0 ; maximum radius
	
	volume#=(b-distance)/(b-a)
	
	If volume<=0 Then 
		FreeEntity bentity
		Return
	EndIf
	
	If volume>1 Then volume=1
	
	; now, is it bigger?
	
	If volume>SFXSingleVolume(btype) Then SFXSingleVolume(btype)=volume
	
	FreeEntity bentity
	
End Function