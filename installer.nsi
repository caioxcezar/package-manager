Name "Package Manager"
OutFile "installer.exe"
InstallDir "$PROGRAMFILES\Package Manager"
RequestExecutionLevel admin
SetCompress auto
SetCompressor lzma

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
    SetOutPath $INSTDIR
    File /r "C:\gtk-build\gtk\x64\release\bin\*.*"

    ; Create shortcut with runas verb to force admin execution
    CreateShortCut "$DESKTOP\Package Manager.lnk" "$INSTDIR\package-manager.exe"
    ; Apply the runas verb to the shortcut
    ShellLink::SetRunAs "$DESKTOP\Package Manager.lnk"
    
    CreateDirectory "$SMPROGRAMS\Package Manager"
    CreateShortCut "$SMPROGRAMS\Package Manager\Package Manager.lnk" "$INSTDIR\package-manager.exe"
    ; Apply the runas verb to the Start Menu shortcut
    ShellLink::SetRunAs "$SMPROGRAMS\Package Manager\Package Manager.lnk"
    
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$DESKTOP\Package Manager.lnk"
    Delete "$SMPROGRAMS\Package Manager\Package Manager.lnk"
    RMDir "$SMPROGRAMS\Package Manager"
    Delete "$INSTDIR\package-manager.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR"
SectionEnd

; Needed for ShellLink plugin
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "TextFunc.nsh"
!include "WinMessages.nsh"
!include "x64.nsh"