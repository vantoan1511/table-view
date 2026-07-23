import { copyFileSync, readFileSync, writeFileSync } from 'fs';
import innosetupCompiler from 'innosetup-compiler';
import { join } from 'path';

console.log('Preparing to build installer...');

// Read arguments for version override
const args = process.argv.slice(2);
let appVersionOverride = null;
if (args.length > 0) {
  appVersionOverride = args[0];
}

// Read neutralino.config.json
const configStr = readFileSync('neutralino.config.json', 'utf8');
const config = JSON.parse(configStr);

const appName = config.applicationName || 'Table View';
const appVersion = appVersionOverride || config.version || '1.0.0';
const appId = 'D1A8F9B2-3C4D-5E6F-7A8B-9C0D1E2F3A4B';

// Copy icon
const iconSrc = join('public', 'favicon.ico');
const iconDest = join('dist', 'table-view', 'icon.ico');
console.log(`Copying icon from ${iconSrc} to ${iconDest}...`);
copyFileSync(iconSrc, iconDest);

// Generate ISS script
const issContent = `
[Setup]
AppId={{${appId}}
AppName=${appName}
AppVersion=${appVersion}
AppPublisher=${config.author || 'Toan Nguyen'}
AppPublisherURL=https://github.com/vantoan1511/table-view
AppSupportURL=https://github.com/vantoan1511/table-view
AppUpdatesURL=https://github.com/vantoan1511/table-view
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DefaultDirName={localappdata}\\${appName}
DisableProgramGroupPage=yes
DisableDirPage=no
PrivilegesRequired=lowest
OutputDir=Output
OutputBaseFilename=setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
SetupIconFile=dist\\table-view\\icon.ico

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "dist\\table-view\\table-view-win_x64.exe"; DestDir: "{app}"; DestName: "table-view.exe"; Flags: ignoreversion
Source: "dist\\table-view\\resources.neu"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\\table-view\\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\\table-view\\bin\\db-bridge.exe"; DestDir: "{app}\\bin"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\\${appName}"; Filename: "{app}\\table-view.exe"; IconFilename: "{app}\\icon.ico"
Name: "{autodesktop}\\${appName}"; Filename: "{app}\\table-view.exe"; IconFilename: "{app}\\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\\table-view.exe"; Description: "{cm:LaunchProgram,${appName}}"; Flags: nowait postinstall skipifsilent
`;

const issPath = 'installer.iss';
writeFileSync(issPath, issContent);

console.log('Compiling installer...');
innosetupCompiler(issPath, { gui: false }, (error) => {
  if (error) {
    console.error('Failed to compile installer:', error);
    process.exit(1);
  }
  console.log('Installer built successfully!');
});
