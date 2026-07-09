import {
  createSignal,
  Show,
  For,
  onMount,
  onCleanup,
  type Component,
} from 'solid-js'
import type { Account, Group } from '../types'
import {
  Copy,
  Check,
  MoreHorizontal,
  Pencil,
  Trash2,
  FolderInput,
  RefreshCw,
  Zap,
  Loader2,
} from 'lucide-solid'
import { cn, DEFAULT_GROUP_ID } from '../lib/utils'

interface AccountCardProps {
  account: Account
  groups: Group[]
  isActive: boolean
  isSwitching: boolean
  onSwitch: (id: string) => void
  onEdit: (account: Account) => void
  onDelete: (id: string) => void
  onMoveGroup: (accountId: string, groupId: string) => void
}

export const AccountCard: Component<AccountCardProps> = (props) => {
  const [copied, setCopied] = createSignal(false)
  const [showMenu, setShowMenu] = createSignal(false)
  const [showGroupMenu, setShowGroupMenu] = createSignal(false)
  let menuRef: HTMLDivElement | undefined

  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef && !menuRef.contains(e.target as Node)) {
        setShowMenu(false)
        setShowGroupMenu(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    onCleanup(() =>
      document.removeEventListener('mousedown', handleClickOutside),
    )
  })

  const group = () =>
    props.groups.find((g) => g.Id === props.account.GroupId)

  const initial = () => {
    const name = props.account.Username || props.account.Remark || 'U'
    return name.charAt(0).toUpperCase()
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(props.account.Username)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {}
  }

  function handleDelete() {
    if (confirm('确定要删除该账号吗？')) {
      props.onDelete(props.account.Id)
      setShowMenu(false)
    }
  }

  function handleMoveToGroup(groupId: string) {
    props.onMoveGroup(props.account.Id, groupId)
    setShowGroupMenu(false)
    setShowMenu(false)
  }

  return (
    <div
      class={cn(
        'group relative flex flex-col rounded-xl border p-4 transition-all duration-200',
        'bg-white border-gray-100 shadow-sm',
        'hover:border-primary/30 hover:shadow-md',
        'dark:bg-dark-card-bg dark:border-dark-card-border',
        'dark:hover:border-primary/40',
        props.isActive && 'border-l-[3px] border-l-emerald-500',
      )}
    >
      {/* Top row: avatar + username + copy + menu */}
      <div class='flex items-center gap-3'>
        {/* Avatar */}
        <div
          class={cn(
            'flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full text-sm font-semibold',
            'bg-primary/10 text-primary',
          )}
        >
          {initial()}
        </div>

        {/* Username + platform */}
        <div class='min-w-0 flex-1'>
          <div class='flex items-center gap-1.5'>
            <span class='truncate text-sm font-medium text-gray-900 dark:text-dark-text'>
              {props.account.Username}
            </span>
            <button
              onClick={handleCopy}
              class='no-drag flex-shrink-0 text-gray-400 hover:text-primary dark:text-dark-text-secondary dark:hover:text-primary'
              title='复制用户名'
            >
              <Show when={copied()} fallback={<Copy size={13} />}>
                <Check size={13} class='text-primary' />
              </Show>
            </button>
          </div>
        </div>

        {/* Menu button */}
        <div class='relative' ref={menuRef}>
          <button
            onClick={() => setShowMenu((p) => !p)}
            class='no-drag flex h-7 w-7 items-center justify-center rounded-md text-gray-400 opacity-0 transition-opacity hover:bg-gray-100 hover:text-gray-600 group-hover:opacity-100 dark:text-dark-text-secondary dark:hover:bg-gray-600/50 dark:hover:text-white'
          >
            <MoreHorizontal size={16} />
          </button>

          {/* Dropdown menu */}
          <Show when={showMenu()}>
            <div class='absolute right-0 top-full z-20 mt-1 w-40 rounded-lg border border-gray-100 bg-white py-1 shadow-lg dark:bg-dark-card-bg dark:border-dark-card-border'>
              <button
                onClick={() => {
                  props.onEdit(props.account)
                  setShowMenu(false)
                }}
                class='flex w-full items-center gap-2 px-3 py-2 text-xs text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
              >
                <Pencil size={13} /> 编辑信息
              </button>
              <Show
                when={
                  props.groups.filter(
                    (g) => g.Id !== props.account.GroupId && g.Id !== DEFAULT_GROUP_ID,
                  ).length > 0 || props.account.GroupId
                }
              >
                <div class='relative'>
                  <button
                    onClick={() => setShowGroupMenu((p) => !p)}
                    class='flex w-full items-center gap-2 px-3 py-2 text-xs text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
                  >
                    <FolderInput size={13} /> 移动到分组
                  </button>
                  <Show when={showGroupMenu()}>
                    <div class='ml-6 mt-0.5 rounded-lg border border-gray-100 bg-white py-1 shadow-lg dark:bg-dark-card-bg dark:border-dark-card-border'>
                      <For
                        each={props.groups.filter(
                          (g) => g.Id !== props.account.GroupId && g.Id !== DEFAULT_GROUP_ID,
                        )}
                      >
                        {(g) => (
                          <button
                            onClick={() => handleMoveToGroup(g.Id)}
                            class='flex w-full items-center gap-2 px-3 py-1.5 text-xs text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
                          >
                            {g.Name}
                          </button>
                        )}
                      </For>
                      <Show when={props.account.GroupId}>
                        <button
                          onClick={() => handleMoveToGroup('')}
                          class='flex w-full items-center gap-2 px-3 py-1.5 text-xs text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
                        >
                          取消分组
                        </button>
                      </Show>
                    </div>
                  </Show>
                </div>
              </Show>
              <hr class='my-1 border-gray-100 dark:border-dark-card-border' />
              <button
                onClick={handleDelete}
                class='flex w-full items-center gap-2 px-3 py-2 text-xs text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'
              >
                <Trash2 size={13} /> 删除账号
              </button>
            </div>
          </Show>
        </div>
      </div>

      {/* Group + tags */}
      <div class='mt-3 flex flex-wrap items-center gap-1.5'>
        <Show when={group()}>
          <span class='inline-flex items-center gap-1 rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary'>
            <span class='h-1.5 w-1.5 rounded-full bg-primary' />
            {group()!.Name}
          </span>
        </Show>
        <For each={props.account.Tags || []}>
          {(tag) => (
            <span class='inline-flex items-center rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:bg-emerald-900/20 dark:text-emerald-300'>
              {tag}
            </span>
          )}
        </For>
      </div>

      {/* Remark */}
      <Show
        when={
          props.account.Remark &&
          props.account.Remark !== props.account.Username
        }
      >
        <p class='mt-2 text-xs text-gray-500 dark:text-dark-text-secondary line-clamp-2'>
          {props.account.Remark}
        </p>
      </Show>

      {/* Switch button or active indicator */}
      <div class='mt-auto pt-3'>
        <Show
          when={props.isActive}
          fallback={
            <button
              onClick={() => props.onSwitch(props.account.Id)}
              disabled={props.isSwitching}
              class={cn(
                'flex w-full items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all',
                'border border-primary text-primary hover:bg-primary hover:text-white',
                'active:scale-[0.98]',
                'disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-transparent disabled:hover:text-primary',
              )}
            >
              {props.isSwitching ? (
                <>
                  <Loader2 size={14} class='animate-spin' />
                  切换中...
                </>
              ) : (
                <>
                  <RefreshCw size={14} />
                  使用此账号
                </>
              )}
            </button>
          }
        >
          <div class='flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-50 px-4 py-2 text-sm font-semibold text-emerald-600 dark:bg-emerald-800/40 dark:text-emerald-300'>
            <Zap size={14} />
            使用中
          </div>
        </Show>
      </div>
    </div>
  )
}
