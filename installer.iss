#define MyAppName "Tol"
#define MyAppVersion "0.1.0"
#define MyAppExeName "tol.exe"

[Setup]
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=Output
OutputBaseFilename=TolSetup_v{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
ChangesEnvironment=yes
PrivilegesRequired=lowest

[Files]
Source: "target\release\tol-lang.exe"; DestDir: "{app}"; DestName: "tol.exe"; Flags: ignoreversion
Source: "stdlib\*"; DestDir: "{app}\stdlib"; Flags: recursesubdirs createallsubdirs ignoreversion

[Icons]
Name: "{group}\Tol"; Filename: "{app}\tol.exe"

[Registry]
Root: HKCU; \
    Subkey: "Environment"; \
    ValueType: expandsz; \
    ValueName: "TOL_STDLIB"; \
    ValueData: "{app}\stdlib"; \
    Flags: uninsdeletevalue

Root: HKCU; \
    Subkey: "Environment"; \
    ValueType: expandsz; \
    ValueName: "Path"; \
    ValueData: "{olddata};{app}"; \
    Check: NeedsAddPath(ExpandConstant('{app}'))

[Code]
function NeedsAddPath(Dir: string): Boolean;
var
  Path: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', Path) then
    Path := '';

  Result := Pos(';' + UpperCase(Dir) + ';',
    ';' + UpperCase(Path) + ';') = 0;
end;

const
  WM_SETTINGCHANGE = $001A;
  SMTO_ABORTIFHUNG = $0002;

function SendMessageTimeout(
  hWnd: Integer;
  Msg: Integer;
  wParam: Integer;
  lParam: String;
  fuFlags: Integer;
  uTimeout: Integer;
  out lpdwResult: Integer
): Integer;
external 'SendMessageTimeoutW@user32.dll stdcall';

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssPostInstall then
    SendMessageTimeout(
      HWND_BROADCAST,
      WM_SETTINGCHANGE,
      0,
      'Environment',
      SMTO_ABORTIFHUNG,
      5000,
      ResultCode);
end;
