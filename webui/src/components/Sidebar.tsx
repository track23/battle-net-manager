import {
  createSignal,
  For,
  Show,
  onMount,
  onCleanup,
  type Component,
} from 'solid-js'
import type { Group } from '../types'
import {
  Users,
  ChevronDown,
  ChevronRight,
  Tag,
  Plus,
  Settings,
  Moon,
  Sun,
  Menu,
  X,
} from 'lucide-solid'
import { DEFAULT_GROUP_ID } from '../lib/utils'
import { cn } from '../lib/utils'
import { useI18n } from '../i18n'

interface SidebarProps {
  groups: Group[]
  groupCounts: Map<string, number>
  allTags: string[]
  selectedGroupId: string | null
  selectedTag: string | null
  expandedGroupIds: Set<string>
  isDarkMode: boolean
  isOpen: boolean
  onSelectGroup: (id: string | null) => void
  onSelectTag: (tag: string | null) => void
  onToggleExpanded: (id: string) => void
  onCreateGroup: (name: string) => void
  onRenameGroup: (id: string, name: string) => void
  onDeleteGroup: (id: string) => void
  onToggleDarkMode: () => void
  onToggleSidebar: () => void
}

export const Sidebar: Component<SidebarProps> = (props) => {
  const { t } = useI18n()
  const [newGroupName, setNewGroupName] = createSignal('')
  const [showNewGroupInput, setShowNewGroupInput] = createSignal(false)
  const [editingGroupId, setEditingGroupId] = createSignal<string | null>(null)
  const [editGroupName, setEditGroupName] = createSignal('')
  const [contextMenuId, setContextMenuId] = createSignal<string | null>(null)

  // Close context menu on outside click
  onMount(() => {
    const handler = () => setContextMenuId(null)
    document.addEventListener('click', handler)
    onCleanup(() => document.removeEventListener('click', handler))
  })

  const totalAccounts = () => {
    let count = 0
    for (const c of props.groupCounts.values()) count += c
    return count
  }

  function handleCreateGroup() {
    const name = newGroupName().trim()
    if (name) {
      props.onCreateGroup(name)
      setNewGroupName('')
      setShowNewGroupInput(false)
    }
  }

  function startEditGroup(id: string, name: string) {
    setEditingGroupId(id)
    setEditGroupName(name)
    setContextMenuId(null)
  }

  function handleRenameGroup() {
    const id = editingGroupId()
    if (id) {
      props.onRenameGroup(id, editGroupName())
      setEditingGroupId(null)
    }
  }

  function handleDeleteGroup(id: string) {
    setContextMenuId(null)
    if (confirm(t('deleteGroupConfirm'))) {
      props.onDeleteGroup(id)
    }
  }

  return (
    <>
      {/* Mobile overlay */}
      <Show when={props.isOpen}>
        <div
          class='fixed inset-0 z-30 bg-black/50 lg:hidden'
          onClick={props.onToggleSidebar}
        />
      </Show>

      <aside
        class={cn(
          'fixed inset-y-0 left-0 z-40 flex w-64 flex-col border-r border-gray-100 transition-transform duration-200',
          'bg-gray-50 dark:bg-dark-sidebar-bg dark:border-dark-sidebar-border',
          'lg:relative lg:translate-x-0',
          props.isOpen ? 'translate-x-0' : '-translate-x-full',
        )}
      >
        {/* Navigation */}
        <nav class='flex-1 overflow-y-auto px-3 py-2 hide-scrollbar'>
          {/* All accounts */}
          <button
            onClick={() => props.onSelectGroup(null)}
            class={cn(
              'flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
              props.selectedGroupId === null && !props.selectedTag
                ? 'bg-primary/10 text-primary font-medium'
                : 'text-gray-600 hover:bg-gray-100 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
            )}
          >
            <Users size={18} />
            <span class='flex-1 text-left'>{t('allAccounts')}</span>
            <span class='text-xs text-gray-400 dark:text-dark-text-secondary'>
              {totalAccounts()}
            </span>
          </button>

          {/* Groups section */}
          <div class='mt-4'>
            <div class='flex items-center justify-between px-3 py-1'>
              <span class='text-xs font-medium text-gray-400 uppercase dark:text-dark-text-secondary'>
                {t('groups')}
              </span>
              <button
                onClick={() => setShowNewGroupInput(true)}
                class='text-gray-400 hover:text-primary dark:text-dark-text-secondary dark:hover:text-primary'
              >
                <Plus size={14} />
              </button>
            </div>

            <Show when={showNewGroupInput()}>
              <div
                class='flex items-center gap-1 px-2 py-1'
                onFocusOut={(e) => {
                  if (
                    !(e.currentTarget as HTMLElement).contains(
                      e.relatedTarget as Node | null,
                    )
                  ) {
                    setShowNewGroupInput(false)
                  }
                }}
              >
                <input
                  type='text'
                  value={newGroupName()}
                  onInput={(e) => setNewGroupName(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleCreateGroup()
                    if (e.key === 'Escape') setShowNewGroupInput(false)
                  }}
                  placeholder={t('newGroupName')}
                  class='flex-1 rounded-md border border-gray-100 px-2 py-1 text-xs bg-white dark:bg-dark-card-bg dark:border-dark-card-border dark:text-dark-text'
                  ref={(el) => queueMicrotask(() => el.focus())}
                />
              </div>
            </Show>

            <For each={props.groups.filter((g) => g.Id !== DEFAULT_GROUP_ID)}>
              {(group) => (
                <div class='relative'>
                  <Show
                    when={editingGroupId() !== group.Id}
                    fallback={
                      <div
                        class='flex items-center gap-1 px-2 py-1'
                        onFocusOut={(e) => {
                          if (
                            !(e.currentTarget as HTMLElement).contains(
                              e.relatedTarget as Node | null,
                            )
                          ) {
                            setEditingGroupId(null)
                          }
                        }}
                      >
                        <input
                          type='text'
                          value={editGroupName()}
                          onInput={(e) =>
                            setEditGroupName(e.currentTarget.value)
                          }
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') handleRenameGroup()
                            if (e.key === 'Escape') setEditingGroupId(null)
                          }}
                          class='flex-1 rounded-md border border-gray-100 px-2 py-1 text-xs bg-white dark:bg-dark-card-bg dark:border-dark-card-border dark:text-dark-text'
                          ref={(el) => queueMicrotask(() => el.focus())}
                        />
                      </div>
                    }
                  >
                    <button
                      onClick={() => props.onSelectGroup(group.Id)}
                      onDblClick={() => startEditGroup(group.Id, group.Name)}
                      onContextMenu={(e) => {
                        e.preventDefault()
                        setContextMenuId(
                          contextMenuId() === group.Id ? null : group.Id,
                        )
                      }}
                      class={cn(
                        'flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
                        props.selectedGroupId === group.Id
                          ? 'bg-primary/10 text-primary font-medium'
                          : 'text-gray-600 hover:bg-gray-100 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                      )}
                    >
                      <div class='h-2.5 w-2.5 rounded-full bg-primary' />
                      <span class='flex-1 text-left'>{group.Name}</span>
                      <span class='text-xs text-gray-400 dark:text-dark-text-secondary'>
                        {props.groupCounts.get(group.Id) || 0}
                      </span>
                    </button>
                  </Show>

                  {/* Context menu */}
                  <Show when={contextMenuId() === group.Id}>
                    <div
                      class='absolute right-0 top-full z-10 mt-1 w-32 rounded-lg border border-gray-100 bg-white py-1 shadow-lg dark:bg-dark-card-bg dark:border-dark-card-border'
                      onClick={(e) => e.stopPropagation()}
                    >
                      <button
                        onClick={() => startEditGroup(group.Id, group.Name)}
                        class='flex w-full items-center gap-2 px-3 py-1.5 text-xs text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
                      >
                        <Settings size={12} /> {t('rename')}
                      </button>
                      <button
                        onClick={() => handleDeleteGroup(group.Id)}
                        class='flex w-full items-center gap-2 px-3 py-1.5 text-xs text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'
                      >
                        {t('deleteGroup')}
                      </button>
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>

          {/* Tags section */}
          <Show when={props.allTags.length > 0}>
            <div class='mt-4'>
              <div class='px-3 py-1'>
                <span class='text-xs font-medium text-gray-400 uppercase dark:text-dark-text-secondary'>
                  {t('tags')}
                </span>
              </div>
              <For each={props.allTags}>
                {(tag) => (
                  <button
                    onClick={() =>
                      props.onSelectTag(props.selectedTag === tag ? null : tag)
                    }
                    class={cn(
                      'flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
                      props.selectedTag === tag
                        ? 'bg-primary/10 text-primary font-medium'
                        : 'text-gray-600 hover:bg-gray-100 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                    )}
                  >
                    <Tag size={16} />
                    <span class='flex-1 text-left'>{tag}</span>
                  </button>
                )}
              </For>
            </div>
          </Show>
        </nav>

        {/* Footer - dark mode toggle */}
        <div class='border-t border-gray-100 px-3 py-3 dark:border-dark-sidebar-border'>
          <button
            onClick={props.onToggleDarkMode}
            class='flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm text-gray-600 hover:bg-gray-100 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
          >
            <Show when={props.isDarkMode} fallback={<Sun size={18} />}>
              <Moon size={18} />
            </Show>
            <span>{props.isDarkMode ? t('lightMode') : t('darkMode')}</span>
          </button>
        </div>
      </aside>
    </>
  )
}
