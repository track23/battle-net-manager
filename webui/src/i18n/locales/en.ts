import type { TranslationKey } from './zh'

export const en: Record<TranslationKey, string> = {
  // Common
  loading: 'Loading...',
  cancel: 'Cancel',
  back: 'Back',
  close: 'Close',
  rename: 'Rename',
  saving: 'Saving...',

  // Sidebar
  allAccounts: 'All Accounts',
  groups: 'Groups',
  tags: 'Tags',
  newGroupName: 'New group name',
  deleteGroup: 'Delete Group',
  deleteGroupConfirm: 'Delete this group? Accounts will be moved to ungrouped.',
  lightMode: 'Light Mode',
  darkMode: 'Dark Mode',

  // Header
  searchPlaceholder: 'Search accounts, tags, or remarks...',
  addAccount: 'Add Account',

  // AccountGrid
  noAccounts: 'No Accounts',
  noAccountsHint: 'Click "Add Account" in the top right to get started',

  // AccountCard
  deleteAccountConfirm: 'Delete this account?',
  copyUsername: 'Copy Username',
  editInfo: 'Edit Info',
  moveToGroup: 'Move to Group',
  ungroup: 'Ungroup',
  deleteAccount: 'Delete Account',
  switching: 'Switching...',
  useThisAccount: 'Use This Account',
  inUse: 'In Use',

  // AccountModal
  noGroup: 'No Group',
  enterBattleTag: 'Please enter Battle.net ID',
  operationFailed: 'Operation failed, please try again',
  selectAction: 'Select Action',
  editAccount: 'Edit Account',
  addAccountTitle: 'Add Account',
  selectActionDesc: 'Choose the action you want to perform',
  editAccountDesc: 'Modify account info, group can be changed anytime.',
  addAccountDesc: 'Fill in account info, group can be changed anytime.',
  saveCurrentState: 'Save Current State',
  saveCurrentStateDesc: 'Extract the currently logged-in Battle.net config and save permanently',
  loginNewAccount: 'Login New Account',
  loginNewAccountDesc: 'Force close Battle.net so you can enter new credentials',
  battleTag: 'Battle.net ID',
  battleTagPlaceholder: 'e.g. Player#1234',
  belongsToGroup: 'Group',
  saveToGroup: 'Save to Group',
  optionalPressEnter: '(Optional, press Enter to add)',
  enterTagName: 'Enter tag name',
  accountRemark: 'Remark',
  remarkPlaceholder: 'e.g. Unranked',
  saveChanges: 'Save Changes',
  confirmSave: 'Confirm Save',

  // App.tsx
  saveFailed: 'Save failed, please try again',
  saveFailedLogin: 'Save failed, please make sure you have logged in via the Battle.net client',

  // UpdateModal
  newVersion: 'New Version Found',
  updateAvailable: 'A new update is available',
  newVersionLabel: 'New version',
  releaseDate: 'Released',
  updateNotes: 'Release Notes',
  updateDownloaded: 'Update downloaded, the app will restart to finish installation',
  updateLater: 'Update Later',
  downloading: 'Downloading...',
  updateNow: 'Update Now',
  installFailed: 'Installation failed, please try again or download manually',
}
