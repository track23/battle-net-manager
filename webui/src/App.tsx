import {
  createSignal,
  createEffect,
  createMemo,
  onMount,
  Show,
  For,
  type Component,
} from 'solid-js'
import type { Account, Group, UpdateInfo } from './types'
import { getBridge } from './bridge'
import {
  EXPANDED_GROUPS_KEY,
  DARK_MODE_KEY,
  compareByLastUsed,
  toLocalISOString,
} from './lib/utils'
import { Sidebar } from './components/Sidebar'
import { Header } from './components/Header'
import { AccountGrid } from './components/AccountGrid'
import { AccountModal } from './components/AccountModal'
import { UpdateModal } from './components/UpdateModal'

const App: Component = () => {
  // ── Core data ──
  const [accounts, setAccounts] = createSignal<Account[]>([])
  const [groups, setGroups] = createSignal<Group[]>([])
  const [loading, setLoading] = createSignal(true)
  const [activeAccountId, setActiveAccountId] = createSignal<string | null>(null)

  // ── UI state ──
  const [isDarkMode, setIsDarkMode] = createSignal(false)
  const [sidebarOpen, setSidebarOpen] = createSignal(false)
  const [selectedGroupId, setSelectedGroupId] = createSignal<string | null>(
    null,
  )
  const [selectedTag, setSelectedTag] = createSignal<string | null>(null)
  const [searchQuery, setSearchQuery] = createSignal('')
  const [expandedGroupIds, setExpandedGroupIds] = createSignal<Set<string>>(
    new Set(),
  )

  // ── Modal state ──
  const [modalOpen, setModalOpen] = createSignal(false)
  const [editingAccount, setEditingAccount] = createSignal<Account | null>(null)

  // ── Update state ──
  const [updateInfo, setUpdateInfo] = createSignal<UpdateInfo | null>(null)
  const [updateModalOpen, setUpdateModalOpen] = createSignal(false)

  // ── Derived data ──
  const orderedGroups = createMemo(() => {
    return [...groups()].sort((a, b) => a.Name.localeCompare(b.Name))
  })

  const accountsByGroup = createMemo(() => {
    const map = new Map<string, Account[]>()
    for (const account of accounts()) {
      const gid = account.GroupId || ''
      if (!map.has(gid)) map.set(gid, [])
      map.get(gid)!.push(account)
    }
    // Sort by LastUsed
    for (const list of map.values()) {
      list.sort(compareByLastUsed)
    }
    return map
  })

  const allTags = createMemo(() => {
    const tagSet = new Set<string>()
    for (const account of accounts()) {
      if (account.Tags) {
        for (const tag of account.Tags) {
          if (tag) tagSet.add(tag)
        }
      }
    }
    return Array.from(tagSet).sort()
  })

  const filteredAccounts = createMemo(() => {
    let result = accounts()
    const groupId = selectedGroupId()
    const tag = selectedTag()
    const query = searchQuery().toLowerCase()

    if (groupId) {
      result = result.filter((a) => a.GroupId === groupId)
    }

    if (tag) {
      result = result.filter((a) => a.Tags && a.Tags.includes(tag))
    }

    if (query) {
      result = result.filter(
        (a) =>
          a.Username.toLowerCase().includes(query) ||
          a.Remark.toLowerCase().includes(query),
      )
    }

    return result.sort(compareByLastUsed)
  })

  const groupCounts = createMemo(() => {
    const map = new Map<string, number>()
    for (const account of accounts()) {
      const gid = account.GroupId || ''
      map.set(gid, (map.get(gid) || 0) + 1)
    }
    return map
  })

  // ── Dark mode ──
  createEffect(() => {
    const dark = isDarkMode()
    if (dark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
    localStorage.setItem(DARK_MODE_KEY, JSON.stringify(dark))
  })

  // ── Load expanded groups ──
  onMount(() => {
    try {
      const saved = localStorage.getItem(EXPANDED_GROUPS_KEY)
      if (saved) {
        const arr: string[] = JSON.parse(saved)
        setExpandedGroupIds(new Set(arr))
      }
    } catch {}
  })

  // ── Save expanded groups ──
  createEffect(() => {
    const ids = expandedGroupIds()
    localStorage.setItem(EXPANDED_GROUPS_KEY, JSON.stringify(Array.from(ids)))
  })

  // ── Initialize ──
  onMount(async () => {
    // Load dark mode preference
    try {
      const saved = localStorage.getItem(DARK_MODE_KEY)
      if (saved !== null) {
        setIsDarkMode(JSON.parse(saved))
      } else {
        setIsDarkMode(window.matchMedia('(prefers-color-scheme: dark)').matches)
      }
    } catch {
      setIsDarkMode(window.matchMedia('(prefers-color-scheme: dark)').matches)
    }

    await loadData()

    // Check for updates after data loads
    checkForUpdate()
  })

  // ── Data loading ──
  async function loadData() {
    setLoading(true)
    try {
      const bridge = getBridge()
      const [accountsJson, groupsJson, activeId] = await Promise.all([
        bridge.GetAccounts(),
        bridge.GetGroups(),
        bridge.GetActiveAccountId(),
      ])
      const loadedAccounts: Account[] = JSON.parse(accountsJson)
      const loadedGroups: Group[] = JSON.parse(groupsJson)

      setAccounts(loadedAccounts)
      setGroups(loadedGroups)
      setActiveAccountId(activeId)
    } catch (e) {
      console.error('Failed to load data:', e)
    } finally {
      setLoading(false)
    }
  }

  // ── Update checking ──
  async function checkForUpdate() {
    try {
      const bridge = getBridge()
      const update = await bridge.CheckUpdate()
      if (update) {
        setUpdateInfo(update)
        setUpdateModalOpen(true)
      }
    } catch (e) {
      console.error('Failed to check for update:', e)
    }
  }

  async function installUpdate() {
    try {
      const bridge = getBridge()
      await bridge.InstallUpdate()
    } catch (e) {
      console.error('Failed to install update:', e)
      throw e
    }
  }

  // ── Group operations ──
  async function createGroup(name: string) {
    if (!name.trim()) return
    try {
      const bridge = getBridge()
      const json = await bridge.CreateGroup(name.trim())
      const newGroup: Group = JSON.parse(json)
      setGroups((prev) => [...prev, newGroup])
      toggleGroupExpanded(newGroup.Id)
    } catch (e) {
      console.error('Failed to create group:', e)
    }
  }

  async function renameGroup(id: string, name: string) {
    if (!name.trim()) return
    try {
      const bridge = getBridge()
      const ok = await bridge.RenameGroup(id, name.trim())
      if (ok) {
        setGroups((prev) =>
          prev.map((g) => (g.Id === id ? { ...g, Name: name.trim() } : g)),
        )
      }
    } catch (e) {
      console.error('Failed to rename group:', e)
    }
  }

  async function deleteGroup(id: string) {
    if (!id) return
    try {
      const bridge = getBridge()
      const ok = await bridge.DeleteGroup(id)
      if (ok) {
        setGroups((prev) => prev.filter((g) => g.Id !== id))
        setAccounts((prev) =>
          prev.map((a) =>
            a.GroupId === id ? { ...a, GroupId: '' } : a,
          ),
        )
        if (selectedGroupId() === id) setSelectedGroupId(null)
      }
    } catch (e) {
      console.error('Failed to delete group:', e)
    }
  }

  async function moveAccountToGroup(accountId: string, groupId: string) {
    try {
      const bridge = getBridge()
      const ok = await bridge.MoveAccountToGroup(accountId, groupId)
      if (ok) {
        setAccounts((prev) =>
          prev.map((a) =>
            a.Id === accountId ? { ...a, GroupId: groupId } : a,
          ),
        )
      }
    } catch (e) {
      console.error('Failed to move account to group:', e)
    }
  }

  // ── Account operations ──
  async function saveAccount(
    remark: string,
    battleTag: string,
    groupId: string,
    tags: string[],
  ): Promise<string | null> {
    try {
      const bridge = getBridge()
      const editing = editingAccount()
      if (editing) {
        // Edit mode
        const ok = await bridge.UpdateAccountInfo(
          editing.Id,
          remark,
          battleTag,
          tags,
        )
        if (ok) {
          // Handle group change
          if (editing.GroupId !== groupId) {
            await bridge.MoveAccountToGroup(editing.Id, groupId)
          }
          setAccounts((prev) =>
            prev.map((a) =>
              a.Id === editing.Id
                ? { ...a, Remark: remark, Username: battleTag, Tags: tags, GroupId: groupId }
                : a,
            ),
          )
          return null
        }
        return '保存失败，请重试'
      } else {
        // Add mode - save current account
        const ok = await bridge.SaveCurrentAccountToGroup(
          remark,
          battleTag,
          groupId,
          tags,
        )
        if (ok) {
          await loadData()
          return null
        }
        return '保存失败，请确认您已在战网客户端完成登录'
      }
    } catch (e) {
      console.error('Failed to save account:', e)
      return '保存失败，请重试'
    }
  }

  async function loginNewAccount() {
    try {
      const bridge = getBridge()
      await bridge.AddNewAccount()
    } catch (e) {
      console.error('Failed to add new account:', e)
    }
  }

  async function switchAccount(id: string) {
    try {
      const bridge = getBridge()
      await bridge.SwitchAccount(id)
      // Update LastUsed locally
      setAccounts((prev) =>
        prev.map((a) =>
          a.Id === id ? { ...a, LastUsed: toLocalISOString(new Date()) } : a,
        ),
      )
      // Mark this account as active
      setActiveAccountId(id)
    } catch (e) {
      console.error('Failed to switch account:', e)
    }
  }

  async function deleteAccount(id: string) {
    try {
      const bridge = getBridge()
      await bridge.DeleteAccount(id)
      setAccounts((prev) => prev.filter((a) => a.Id !== id))
    } catch (e) {
      console.error('Failed to delete account:', e)
    }
  }

  // ── UI helpers ──
  function toggleGroupExpanded(id: string) {
    setExpandedGroupIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  function openAddModal() {
    setEditingAccount(null)
    setModalOpen(true)
  }

  function openEditModal(account: Account) {
    setEditingAccount(account)
    setModalOpen(true)
  }

  function closeModal() {
    setModalOpen(false)
    setEditingAccount(null)
  }

  // ── Render ──
  return (
    <Show
      when={!loading()}
      fallback={
        <div class='flex h-screen items-center justify-center bg-white dark:bg-dark-page-bg'>
          <div class='flex flex-col items-center gap-3'>
            <div class='h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent' />
            <span class='text-sm text-gray-500 dark:text-dark-text-secondary'>
              加载中...
            </span>
          </div>
        </div>
      }
    >
      <div class='flex h-screen overflow-hidden bg-white dark:bg-dark-page-bg'>
        {/* Sidebar */}
        <Sidebar
          groups={orderedGroups()}
          groupCounts={groupCounts()}
          allTags={allTags()}
          selectedGroupId={selectedGroupId()}
          selectedTag={selectedTag()}
          expandedGroupIds={expandedGroupIds()}
          isDarkMode={isDarkMode()}
          isOpen={sidebarOpen()}
          onSelectGroup={setSelectedGroupId}
          onSelectTag={setSelectedTag}
          onToggleExpanded={toggleGroupExpanded}
          onCreateGroup={createGroup}
          onRenameGroup={renameGroup}
          onDeleteGroup={deleteGroup}
          onToggleDarkMode={() => setIsDarkMode((p) => !p)}
          onToggleSidebar={() => setSidebarOpen((p) => !p)}
        />

        {/* Main content */}
        <div class='flex flex-1 flex-col overflow-hidden'>
          <Header
            searchQuery={searchQuery()}
            totalAccounts={accounts().length}
            onSearch={setSearchQuery}
            onAddAccount={openAddModal}
            onToggleSidebar={() => setSidebarOpen((p) => !p)}
          />

          <AccountGrid
            accounts={filteredAccounts()}
            groups={groups()}
            activeAccountId={activeAccountId()}
            onSwitch={switchAccount}
            onEdit={openEditModal}
            onDelete={deleteAccount}
            onMoveGroup={moveAccountToGroup}
          />
        </div>

        {/* Modal */}
        <Show when={modalOpen()}>
          <AccountModal
            account={editingAccount()}
            groups={orderedGroups()}
            allTags={allTags()}
            onClose={closeModal}
            onSave={saveAccount}
            onLoginNew={loginNewAccount}
          />
        </Show>

        {/* Update Modal */}
        <Show when={updateModalOpen() && updateInfo()}>
          <UpdateModal
            updateInfo={updateInfo()!}
            onClose={() => setUpdateModalOpen(false)}
            onInstall={installUpdate}
          />
        </Show>
      </div>
    </Show>
  )
}

export default App
