import { createSignal, createEffect, For, Show, type Component } from 'solid-js'
import type { Account, Group } from '../types'
import { X, ChevronDown, Sparkles, PlusCircle } from 'lucide-solid'
import { cn, DEFAULT_GROUP_ID } from '../lib/utils'

type ModalMode = 'select' | 'save' | 'edit'

interface AccountModalProps {
  account: Account | null
  groups: Group[]
  allTags: string[]
  onClose: () => void
  onSave: (
    remark: string,
    battleTag: string,
    groupId: string,
    tags: string[],
  ) => Promise<string | null>
  onLoginNew: () => void
}

export const AccountModal: Component<AccountModalProps> = (props) => {
  const [mode, setMode] = createSignal<ModalMode>('select')
  const [remark, setRemark] = createSignal('')
  const [battleTag, setBattleTag] = createSignal('')
  const [groupId, setGroupId] = createSignal('')
  const [showGroupDropdown, setShowGroupDropdown] = createSignal(false)
  const [error, setError] = createSignal('')
  const [tags, setTags] = createSignal<string[]>([])
  const [tagInput, setTagInput] = createSignal('')
  const [showTagSuggestions, setShowTagSuggestions] = createSignal(false)

  const isEditing = () => props.account !== null

  // Track account Id to avoid re-running when the parent re-renders and
  // produces a new object reference for the same account.
  let lastAccountId: string | null = null
  createEffect(() => {
    const id = props.account?.Id ?? null
    if (id === lastAccountId) return
    lastAccountId = id
    if (props.account) {
      setMode('edit')
      setRemark(props.account.Remark || '')
      setBattleTag(props.account.Username || '')
      setGroupId(props.account.GroupId || '')
      setTags(props.account.Tags || [])
    } else {
      setMode('select')
    }
  })

  function resetForm() {
    setRemark('')
    setBattleTag('')
    setGroupId('')
    setError('')
    setTags([])
    setTagInput('')
  }

  const selectedGroupName = () => {
    const g = props.groups.find((g) => g.Id === groupId())
    return g?.Name || '不指定分组'
  }

  function addTag(tag: string) {
    const trimmed = tag.trim()
    if (trimmed && !tags().includes(trimmed)) {
      setTags((prev) => [...prev, trimmed])
    }
    setTagInput('')
    setShowTagSuggestions(false)
  }

  function removeTag(tag: string) {
    setTags((prev) => prev.filter((t) => t !== tag))
  }

  function handleTagKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      addTag(tagInput())
    } else if (
      e.key === 'Backspace' &&
      tagInput() === '' &&
      tags().length > 0
    ) {
      setTags((prev) => prev.slice(0, -1))
    }
  }

  const tagSuggestions = () => {
    const input = tagInput().toLowerCase()
    if (!input) return []
    return props.allTags
      .filter((t) => t.toLowerCase().includes(input) && !tags().includes(t))
      .slice(0, 5)
  }

  const [saving, setSaving] = createSignal(false)

  async function handleSubmit(e: Event) {
    e.preventDefault()
    setError('')
    setSaving(true)
    try {
      const finalRemark = remark().trim()
      const err = await props.onSave(
        finalRemark,
        battleTag().trim(),
        groupId(),
        tags(),
      )
      if (err) {
        setError(err)
      } else {
        props.onClose()
      }
    } catch {
      setError('操作失败，请重试')
    } finally {
      setSaving(false)
    }
  }

  function handleLoginNew() {
    props.onLoginNew()
    props.onClose()
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      props.onClose()
    }
  }

  function goBack() {
    setMode('select')
    resetForm()
  }

  return (
    <div
      class='fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4'
      onClick={handleOverlayClick}
    >
      <div
        class={cn(
          'w-full max-w-lg rounded-2xl border border-gray-100 bg-white p-6 shadow-2xl',
          'dark:bg-dark-card-bg dark:border-dark-card-border',
          'animate-in fade-in zoom-in-95 duration-200',
        )}
      >
        {/* Header */}
        <div class='mb-6 flex items-center justify-between'>
          <div>
            <h2 class='text-lg font-semibold text-gray-900 dark:text-dark-text'>
              {mode() === 'select'
                ? '选择操作'
                : mode() === 'edit'
                  ? '编辑账号'
                  : '录入账号'}
            </h2>
            <p class='mt-0.5 text-xs text-gray-400 dark:text-dark-text-secondary'>
              {mode() === 'select'
                ? '选择您要执行的操作'
                : mode() === 'edit'
                  ? '修改账号信息，分组可随时修改。'
                  : '填写账号信息，分组可随时修改。'}
            </p>
          </div>
          <button
            onClick={props.onClose}
            class='flex h-8 w-8 items-center justify-center rounded-lg text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
          >
            <X size={18} />
          </button>
        </div>

        {/* Select Action Mode */}
        <Show when={mode() === 'select'}>
          <div class='space-y-3'>
            {/* Option A: Save Current State */}
            <button
              onClick={() => setMode('save')}
              class={cn(
                'flex w-full items-start gap-4 rounded-xl border border-gray-100 p-4 text-left transition-all',
                'hover:border-primary/30 hover:bg-primary/5',
                'dark:border-dark-card-border dark:hover:border-primary/30 dark:hover:bg-primary/5',
              )}
            >
              <div class='flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-emerald-100 dark:bg-emerald-900/20'>
                <Sparkles
                  size={20}
                  class='text-emerald-600 dark:text-emerald-300'
                />
              </div>
              <div>
                <div class='font-medium text-gray-900 dark:text-dark-text'>
                  保存当前状态
                </div>
                <div class='mt-0.5 text-sm text-gray-500 dark:text-dark-text-secondary'>
                  提取当前战网已登录的配置文件并永久保存
                </div>
              </div>
            </button>

            {/* Option B: Login New Account */}
            <button
              onClick={handleLoginNew}
              class={cn(
                'flex w-full items-start gap-4 rounded-xl border border-gray-100 p-4 text-left transition-all',
                'hover:border-primary/30 hover:bg-primary/5',
                'dark:border-dark-card-border dark:hover:border-primary/30 dark:hover:bg-primary/5',
              )}
            >
              <div class='flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-blue-100 dark:bg-blue-900/30'>
                <PlusCircle
                  size={20}
                  class='text-blue-600 dark:text-blue-400'
                />
              </div>
              <div>
                <div class='font-medium text-gray-900 dark:text-dark-text'>
                  前往登录新号
                </div>
                <div class='mt-0.5 text-sm text-gray-500 dark:text-dark-text-secondary'>
                  强制关闭当前战网，让你能够输入新的账密
                </div>
              </div>
            </button>
          </div>
        </Show>

        {/* Save / Edit Form Mode */}
        <Show when={mode() === 'save' || mode() === 'edit'}>
          <form onSubmit={handleSubmit} class='space-y-4'>
            {/* Battle.net ID */}
            <div>
              <label class='mb-1.5 block text-sm font-medium text-gray-700 dark:text-dark-text'>
                战网ID <span class='text-xs text-gray-400'>(选填)</span>
              </label>
              <input
                type='text'
                value={battleTag()}
                onInput={(e) => setBattleTag(e.currentTarget.value)}
                placeholder='例如: Player#1234'
                class={cn(
                  'w-full rounded-lg border border-gray-100 px-3 py-2.5 text-sm outline-none transition-colors font-mono',
                  'placeholder:text-gray-400 placeholder:font-sans focus:border-primary focus:ring-1 focus:ring-primary/20',
                  'bg-white dark:bg-dark-sidebar-bg dark:border-dark-card-border dark:text-dark-text dark:placeholder:text-dark-text-secondary',
                )}
              />
            </div>

            {/* Group */}
            <div>
              <label class='mb-1.5 block text-sm font-medium text-gray-700 dark:text-dark-text'>
                {mode() === 'edit' ? '所属分组' : '保存到分组'}
              </label>
              <div class='relative'>
                <button
                  type='button'
                  onClick={() => setShowGroupDropdown((p) => !p)}
                  class={cn(
                    'flex w-full items-center justify-between rounded-lg border border-gray-100 px-3 py-2.5 text-sm outline-none transition-colors',
                    'bg-white dark:bg-dark-sidebar-bg dark:border-dark-card-border dark:text-dark-text',
                  )}
                >
                  <span>{selectedGroupName()}</span>
                  <ChevronDown
                    size={16}
                    class={cn(
                      'text-gray-400 transition-transform',
                      showGroupDropdown() && 'rotate-180',
                    )}
                  />
                </button>

                <Show when={showGroupDropdown()}>
                  <div class='absolute z-10 mt-1 w-full rounded-lg border border-gray-100 bg-white py-1 shadow-lg dark:bg-dark-card-bg dark:border-dark-card-border'>
                    <button
                      type='button'
                      onClick={() => {
                        setGroupId('')
                        setShowGroupDropdown(false)
                      }}
                      class={cn(
                        'flex w-full items-center gap-2 px-3 py-2 text-sm transition-colors',
                        groupId() === ''
                          ? 'bg-primary/10 text-primary font-medium'
                          : 'text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                      )}
                    >
                      不指定分组
                    </button>
                    <For each={props.groups.filter((g) => g.Id !== DEFAULT_GROUP_ID)}>
                      {(g) => (
                        <button
                          type='button'
                          onClick={() => {
                            setGroupId(g.Id)
                            setShowGroupDropdown(false)
                          }}
                          class={cn(
                            'flex w-full items-center gap-2 px-3 py-2 text-sm transition-colors',
                            groupId() === g.Id
                              ? 'bg-primary/10 text-primary font-medium'
                              : 'text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                          )}
                        >
                          {g.Name}
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>

            {/* Tags */}
            <div>
              <label class='mb-1.5 block text-sm font-medium text-gray-700 dark:text-dark-text'>
                标签{' '}
                <span class='text-xs text-gray-400'>(选填，按 Enter 添加)</span>
              </label>
              <div class='relative'>
                <div
                  class={cn(
                    'flex flex-wrap items-center gap-1.5 rounded-lg border border-gray-100 px-3 py-2 transition-colors',
                    'bg-white dark:bg-dark-sidebar-bg dark:border-dark-card-border',
                  )}
                >
                  <For each={tags()}>
                    {(tag) => (
                      <span class='inline-flex items-center gap-1 rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:bg-emerald-900/20 dark:text-emerald-300'>
                        {tag}
                        <button
                          type='button'
                          onClick={() => removeTag(tag)}
                          class='ml-0.5 text-emerald-500 hover:text-emerald-700 dark:text-emerald-300 dark:hover:text-emerald-200'
                        >
                          <X size={12} />
                        </button>
                      </span>
                    )}
                  </For>
                  <input
                    type='text'
                    value={tagInput()}
                    onInput={(e) => {
                      setTagInput(e.currentTarget.value)
                      setShowTagSuggestions(true)
                    }}
                    onFocus={() => setShowTagSuggestions(true)}
                    onBlur={() => setShowTagSuggestions(false)}
                    onKeyDown={handleTagKeyDown}
                    placeholder={tags().length === 0 ? '输入标签名称' : ''}
                    class='min-w-[80px] flex-1 bg-transparent text-sm outline-none placeholder:text-gray-400 dark:text-dark-text dark:placeholder:text-dark-text-secondary'
                  />
                </div>

                {/* Tag suggestions */}
                <Show
                  when={showTagSuggestions() && tagSuggestions().length > 0}
                >
                  <div class='absolute z-10 mt-1 w-full rounded-lg border border-gray-100 bg-white py-1 shadow-lg dark:bg-dark-card-bg dark:border-dark-card-border'>
                    <For each={tagSuggestions()}>
                      {(tag) => (
                        <button
                          type='button'
                          onMouseDown={(e) => e.preventDefault()}
                          onClick={() => addTag(tag)}
                          class='flex w-full items-center gap-2 px-3 py-2 text-sm text-gray-600 hover:bg-gray-50 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border'
                        >
                          {tag}
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>

            {/* Remark */}
            <div>
              <label class='mb-1.5 block text-sm font-medium text-gray-700 dark:text-dark-text'>
                账号备注
              </label>
              <input
                type='text'
                value={remark()}
                onInput={(e) => setRemark(e.currentTarget.value)}
                placeholder='例如: 未定级'
                class={cn(
                  'w-full rounded-lg border border-gray-100 px-3 py-2.5 text-sm outline-none transition-colors',
                  'placeholder:text-gray-400 focus:border-primary focus:ring-1 focus:ring-primary/20',
                  'bg-white dark:bg-dark-sidebar-bg dark:border-dark-card-border dark:text-dark-text dark:placeholder:text-dark-text-secondary',
                )}
              />
            </div>

            {/* Error message */}
            <Show when={error()}>
              <div class='flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400'>
                {error()}
              </div>
            </Show>

            {/* Actions */}
            <div class='flex justify-between pt-2'>
              <Show when={mode() === 'save'}>
                <button
                  type='button'
                  onClick={goBack}
                  class={cn(
                    'rounded-lg border border-gray-200 px-4 py-2 text-sm font-medium transition-colors',
                    'text-gray-600 hover:bg-gray-50',
                    'dark:border-dark-card-border dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                  )}
                >
                  返回
                </button>
              </Show>
              <Show when={mode() === 'edit'}>
                <div />
              </Show>
              <div class='flex gap-3'>
                <button
                  type='button'
                  onClick={props.onClose}
                  class={cn(
                    'rounded-lg border border-gray-200 px-4 py-2 text-sm font-medium transition-colors',
                    'text-gray-600 hover:bg-gray-50',
                    'dark:border-dark-card-border dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                  )}
                >
                  取消
                </button>
                <button
                  type='submit'
                  disabled={saving()}
                  class={cn(
                    'rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white transition-colors',
                    'hover:bg-primary-hover active:scale-[0.98]',
                    'disabled:opacity-50 disabled:cursor-not-allowed',
                  )}
                >
                  {saving()
                    ? '保存中...'
                    : mode() === 'edit'
                      ? '保存修改'
                      : '确定保存'}
                </button>
              </div>
            </div>
          </form>
        </Show>
      </div>
    </div>
  )
}
