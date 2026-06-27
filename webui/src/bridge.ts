import type { Account, Group, UpdateInfo } from './types'
import { toLocalISOString } from './lib/utils'

// ─── Tauri Bridge ──────────────────────────────────────────────────────

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let tauriInvoke:
  | ((cmd: string, args?: Record<string, unknown>) => Promise<any>)
  | null = null

async function ensureInvoke() {
  if (!tauriInvoke) {
    const { invoke } = await import('@tauri-apps/api/core')
    tauriInvoke = invoke
  }
}

class TauriBridge {
  async GetAccounts(): Promise<string> {
    await ensureInvoke()
    return tauriInvoke!('get_accounts')
  }

  async GetGroups(): Promise<string> {
    await ensureInvoke()
    return tauriInvoke!('get_groups')
  }

  async CreateGroup(name: string): Promise<string> {
    await ensureInvoke()
    return tauriInvoke!('create_group', { name })
  }

  async RenameGroup(id: string, name: string): Promise<boolean> {
    await ensureInvoke()
    return tauriInvoke!('rename_group', { id, name })
  }

  async DeleteGroup(id: string): Promise<boolean> {
    await ensureInvoke()
    return tauriInvoke!('delete_group', { id })
  }

  async MoveAccountToGroup(
    accountId: string,
    groupId: string,
  ): Promise<boolean> {
    await ensureInvoke()
    return tauriInvoke!('move_account_to_group', { accountId, groupId })
  }

  async UpdateAccountInfo(
    accountId: string,
    remark: string,
    battleTag: string,
    tags: string[],
  ): Promise<boolean> {
    await ensureInvoke()
    return tauriInvoke!('update_account_info', {
      accountId,
      remark,
      battleTag,
      tags,
    })
  }

  async SaveCurrentAccountToGroup(
    remark: string,
    battleTag: string,
    groupId: string,
    tags: string[],
  ): Promise<boolean> {
    await ensureInvoke()
    return tauriInvoke!('save_current_account_to_group', {
      remark,
      battleTag,
      groupId,
      tags,
    })
  }

  async SwitchAccount(id: string): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('switch_account', { id })
  }

  async DeleteAccount(id: string): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('delete_account', { id })
  }

  async AddNewAccount(): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('add_new_account')
  }

  async GetActiveAccountId(): Promise<string | null> {
    await ensureInvoke()
    return tauriInvoke!('get_active_account_id')
  }

  async OpenExternalUrl(url: string): Promise<boolean> {
    await ensureInvoke()
    return tauriInvoke!('open_external_url', { url })
  }

  async DragWindow(): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('drag_window')
  }

  async MinimizeApp(): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('minimize_app')
  }

  async CloseApp(): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('close_app')
  }

  async CheckUpdate(): Promise<UpdateInfo | null> {
    await ensureInvoke()
    return tauriInvoke!('check_update')
  }

  async InstallUpdate(): Promise<void> {
    await ensureInvoke()
    await tauriInvoke!('install_update')
  }
}

// ─── Mock Bridge (for browser dev without Tauri) ──────────────────────

class MockBridge {
  private accounts: Account[] = [
    {
      Id: '1',
      Remark: '',
      Username: 'Player#001',
      LastUsed: toLocalISOString(new Date()),
      GroupId: '',
      Tags: [],
    },
    {
      Id: '2',
      Remark: '',
      Username: 'Player#002',
      LastUsed: toLocalISOString(new Date()),
      GroupId: '',
      Tags: [],
    },
  ]
  private groups: Group[] = []

  async GetAccounts(): Promise<string> {
    return JSON.stringify(this.accounts)
  }

  async GetGroups(): Promise<string> {
    return JSON.stringify(this.groups)
  }

  async CreateGroup(name: string): Promise<string> {
    const group = {
      Id: Math.random().toString(36).substring(7),
      Name: name.trim(),
      CreatedAt: new Date().toISOString(),
    }
    this.groups.push(group)
    return JSON.stringify(group)
  }

  async RenameGroup(id: string, name: string): Promise<boolean> {
    if (id === 'default') return false
    const group = this.groups.find((g) => g.Id === id)
    if (!group) return false
    group.Name = name.trim()
    return true
  }

  async DeleteGroup(id: string): Promise<boolean> {
    if (id === 'default') return false
    this.groups = this.groups.filter((g) => g.Id !== id)
    this.accounts = this.accounts.map((a) =>
      a.GroupId === id ? { ...a, GroupId: '' } : a,
    )
    return true
  }

  async MoveAccountToGroup(
    accountId: string,
    groupId: string,
  ): Promise<boolean> {
    const account = this.accounts.find((a) => a.Id === accountId)
    if (!account) return false
    account.GroupId = groupId || ''
    return true
  }

  async UpdateAccountInfo(
    accountId: string,
    remark: string,
    battleTag: string,
    tags: string[],
  ): Promise<boolean> {
    const account = this.accounts.find((a) => a.Id === accountId)
    if (!account) return false
    account.Remark = remark.trim()
    account.Username = battleTag.trim()
    account.Tags = tags
    return true
  }

  async SaveCurrentAccountToGroup(
    remark: string,
    battleTag: string,
    groupId: string,
    tags: string[],
  ): Promise<boolean> {
    this.accounts.push({
      Id: Math.random().toString(36).substring(7),
      Remark: remark,
      Username: battleTag,
      LastUsed: toLocalISOString(new Date()),
      GroupId: groupId || '',
      Tags: tags,
    })
    return true
  }

  async SwitchAccount(id: string): Promise<void> {
    console.log('Switching to account', id)
    const idx = this.accounts.findIndex((a) => a.Id === id)
    if (idx !== -1) {
      this.accounts[idx].LastUsed = toLocalISOString(new Date())
    }
  }

  async DeleteAccount(id: string): Promise<void> {
    this.accounts = this.accounts.filter((a) => a.Id !== id)
  }

  async AddNewAccount(): Promise<void> {
    console.log('Mock: Opening Battle.net to add new account')
  }

  async GetActiveAccountId(): Promise<string | null> {
    return null
  }

  async OpenExternalUrl(url: string): Promise<boolean> {
    window.open(url, '_blank', 'noopener,noreferrer')
    return true
  }

  async CloseApp(): Promise<void> {
    console.log('Mock: Closing application...')
  }

  async MinimizeApp(): Promise<void> {
    console.log('Mock: Minimizing application...')
  }

  async DragWindow(): Promise<void> {
    console.log('Mock: Dragging window')
  }

  async CheckUpdate(): Promise<UpdateInfo | null> {
    console.log('Mock: Checking for updates')
    return null
  }

  async InstallUpdate(): Promise<void> {
    console.log('Mock: Installing update')
  }
}

// ─── Bridge accessor ──────────────────────────────────────────────────

export type Bridge = TauriBridge | MockBridge

let tauriBridgeInstance: TauriBridge | null = null
let mockBridgeInstance: MockBridge | null = null

export const getBridge = (): Bridge => {
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    if (!tauriBridgeInstance) {
      tauriBridgeInstance = new TauriBridge()
    }
    return tauriBridgeInstance
  }

  if (typeof window !== 'undefined') {
    if (!mockBridgeInstance) {
      mockBridgeInstance = new MockBridge()
    }
    return mockBridgeInstance
  }

  return new MockBridge()
}
