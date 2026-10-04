#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

[Setup]
AppId=0xPolybit.Princi
AppName=Princi
AppVersion={#AppVersion}
DefaultDirName={code:GetDefaultInstallDir}
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist
OutputBaseFilename=Princi-Setup-{#AppVersion}-x64
ChangesEnvironment=yes
ChangesAssociations=yes
UninstallDisplayName=Princi Compiler
UninstallDisplayIcon={app}\bin\princi.exe

[Files]
Source: "..\target\release\princi.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Tasks]
Name: "addtopath"; Description: "Add Princi to your user PATH"; GroupDescription: "Command-line integration:"; Flags: checkedonce
Name: "buildcontext"; Description: "Add a Build with Princi action for .prnc and .princi files"; GroupDescription: "Explorer integration:"; Flags: checkedonce

[Registry]
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.prnc\shell\BuildWithPrinci"; ValueType: string; ValueName: "MUIVerb"; ValueData: "Build with Princi"; Flags: uninsdeletekey; Tasks: buildcontext
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.prnc\shell\BuildWithPrinci\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\princi.exe"" build ""%1"""; Flags: uninsdeletekey; Tasks: buildcontext
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.princi\shell\BuildWithPrinci"; ValueType: string; ValueName: "MUIVerb"; ValueData: "Build with Princi"; Flags: uninsdeletekey; Tasks: buildcontext
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.princi\shell\BuildWithPrinci\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\princi.exe"" build ""%1"""; Flags: uninsdeletekey; Tasks: buildcontext

[Code]
function GetDefaultInstallDir(Param: string): string;
var
  LocalAppData: string;
begin
  LocalAppData := GetEnv('LOCALAPPDATA');
  if LocalAppData = '' then
    LocalAppData := ExpandConstant('{userpf}\AppData\Local');
  Result := AddBackslash(LocalAppData) + 'Programs\Princi';
end;

function NormalizePathEntry(Value: string): string;
begin
  Result := Lowercase(Trim(Value));
  StringChangeEx(Result, '/', '\', True);
  while (Length(Result) > 3) and (Copy(Result, Length(Result), 1) = '\') do
    Delete(Result, Length(Result), 1);
end;

function PathContainsEntry(ExistingPath, Entry: string): Boolean;
var
  Remainder, Segment: string;
  Separator, Target: Integer;
begin
  Target := 0;
  Remainder := ExistingPath;
  Result := False;
  while True do
  begin
    Separator := Pos(';', Remainder);
    if Separator = 0 then
    begin
      Segment := Remainder;
      Target := 1;
    end
    else
    begin
      Segment := Copy(Remainder, 1, Separator - 1);
      Delete(Remainder, 1, Separator);
    end;

    if NormalizePathEntry(Segment) = NormalizePathEntry(Entry) then
    begin
      Result := True;
      Exit;
    end;

    if Target = 1 then
      Break;
  end;
end;

function RemovePathEntry(ExistingPath, Entry: string): string;
var
  Remainder, Segment: string;
  Separator, Target: Integer;
  First: Boolean;
begin
  Remainder := ExistingPath;
  Result := '';
  First := True;
  while True do
  begin
    Separator := Pos(';', Remainder);
    if Separator = 0 then
    begin
      Segment := Remainder;
      Target := 1;
    end
    else
    begin
      Segment := Copy(Remainder, 1, Separator - 1);
      Delete(Remainder, 1, Separator);
    end;

    if NormalizePathEntry(Segment) <> NormalizePathEntry(Entry) then
    begin
      if not First then
        Result := Result + ';';
      Result := Result + Segment;
      First := False;
    end;

    if Target = 1 then
      Break;
  end;
end;

procedure AddPrinciToUserPath;
var
  ExistingPath, PrinciBin: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', ExistingPath) then
    ExistingPath := '';
  PrinciBin := ExpandConstant('{app}\bin');
  if not PathContainsEntry(ExistingPath, PrinciBin) then
  begin
    if ExistingPath = '' then
      ExistingPath := PrinciBin
    else
      ExistingPath := ExistingPath + ';' + PrinciBin;
    if not RegWriteExpandStringValue(HKCU, 'Environment', 'Path', ExistingPath) then
      RaiseException('Could not add Princi to the current user PATH.');
  end;
end;

procedure RemovePrinciFromUserPath;
var
  ExistingPath, UpdatedPath, PrinciBin: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', ExistingPath) then
    Exit;
  PrinciBin := ExpandConstant('{app}\bin');
  UpdatedPath := RemovePathEntry(ExistingPath, PrinciBin);
  if NormalizePathEntry(UpdatedPath) = NormalizePathEntry(ExistingPath) then
    Exit;

  if UpdatedPath = '' then
  begin
    if not RegDeleteValue(HKCU, 'Environment', 'Path') then
      Log('Could not remove the empty user PATH value after uninstalling Princi.');
  end
  else if not RegWriteExpandStringValue(HKCU, 'Environment', 'Path', UpdatedPath) then
    Log('Could not remove Princi from the user PATH during uninstall.');
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    AddPrinciToUserPath;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
    RemovePrinciFromUserPath;
end;
