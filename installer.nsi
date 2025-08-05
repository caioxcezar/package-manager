Name "Package Manager"
OutFile "installer.exe"
InstallDir "$PROGRAMFILES\Package Manager"
RequestExecutionLevel admin
SetCompress auto
SetCompressor lzma

!define VERSION "1.11.2"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
    SetOutPath $INSTDIR
    File /r "C:\gtk-build\gtk\x64\release\bin\*.*"

    ; Create Start Menu folder and shortcuts
    CreateDirectory "$SMPROGRAMS\Package Manager"
    CreateShortCut "$SMPROGRAMS\Package Manager\Package Manager.lnk" "$INSTDIR\package-manager.exe"
    CreateShortCut "$SMPROGRAMS\Package Manager\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    
    ; Apply the runas verb to the shortcuts
    ShellLink::SetRunAsAdministrator "$SMPROGRAMS\Package Manager\Package Manager.lnk"
    
    ; Optional desktop shortcut
    CreateShortCut "$DESKTOP\Package Manager.lnk" "$INSTDIR\package-manager.exe"
    ShellLink::SetRunAsAdministrator "$DESKTOP\Package Manager.lnk"
    
    ; Write uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Add basic registry entries for Add/Remove Programs
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "DisplayName" "Package Manager"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "UninstallString" "$\"$INSTDIR\Uninstall.exe$\""
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "DisplayIcon" "$INSTDIR\package-manager.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "Publisher" "Hobby Project"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager" \
                 "NoRepair" 1
SectionEnd

Section "Uninstall"
    ; Remove shortcuts
    Delete "$DESKTOP\Package Manager.lnk"
    Delete "$SMPROGRAMS\Package Manager\Package Manager.lnk"
    Delete "$SMPROGRAMS\Package Manager\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Package Manager"
    
    ; Remove files
    Delete "$INSTDIR\package-manager.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR"
    
    ; Remove registry entries
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\PackageManager"
SectionEnd

!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "TextFunc.nsh"
!include "WinMessages.nsh"
!include "x64.nsh"