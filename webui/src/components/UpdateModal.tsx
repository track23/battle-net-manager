import { createSignal, Show, type Component } from 'solid-js'
import type { UpdateInfo } from '../types'
import { X, Download, CheckCircle, AlertCircle } from 'lucide-solid'
import { cn } from '../lib/utils'

interface UpdateModalProps {
  updateInfo: UpdateInfo
  onClose: () => void
  onInstall: () => Promise<void>
}

export const UpdateModal: Component<UpdateModalProps> = (props) => {
  const [installing, setInstalling] = createSignal(false)
  const [error, setError] = createSignal('')
  const [success, setSuccess] = createSignal(false)

  async function handleInstall() {
    setInstalling(true)
    setError('')
    try {
      await props.onInstall()
      setSuccess(true)
    } catch (e) {
      setError('安装失败，请重试或手动下载更新')
      console.error('Failed to install update:', e)
    } finally {
      setInstalling(false)
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !installing()) {
      props.onClose()
    }
  }

  return (
    <div
      class='fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4'
      onClick={handleOverlayClick}
    >
      <div
        class={cn(
          'w-full max-w-md rounded-2xl border border-gray-100 bg-white p-6 shadow-2xl',
          'dark:bg-dark-card-bg dark:border-dark-card-border',
          'animate-in fade-in zoom-in-95 duration-200',
        )}
      >
        {/* Header */}
        <div class='mb-6 flex items-center justify-between'>
          <div>
            <h2 class='text-lg font-semibold text-gray-900 dark:text-dark-text'>
              发现新版本
            </h2>
            <p class='mt-0.5 text-xs text-gray-400 dark:text-dark-text-secondary'>
              有新的更新可用
            </p>
          </div>
          <button
            onClick={props.onClose}
            disabled={installing()}
            class='flex h-8 w-8 items-center justify-center rounded-lg text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border disabled:opacity-50'
          >
            <X size={18} />
          </button>
        </div>

        {/* Update Info */}
        <div class='space-y-4'>
          {/* Version */}
          <div class='flex items-center gap-3 rounded-lg bg-emerald-50 p-3 dark:bg-emerald-900/20'>
            <CheckCircle
              size={20}
              class='text-emerald-600 dark:text-emerald-400'
            />
            <div>
              <div class='text-sm font-medium text-emerald-800 dark:text-emerald-200'>
                新版本: v{props.updateInfo.version}
              </div>
              <Show when={props.updateInfo.date}>
                <div class='text-xs text-emerald-600 dark:text-emerald-400'>
                  发布时间: {props.updateInfo.date}
                </div>
              </Show>
            </div>
          </div>

          {/* Release Notes */}
          <Show when={props.updateInfo.notes}>
            <div class='rounded-lg border border-gray-100 p-3 dark:border-dark-card-border'>
              <div class='mb-2 text-sm font-medium text-gray-700 dark:text-dark-text'>
                更新内容
              </div>
              <div class='max-h-40 overflow-y-auto text-sm text-gray-600 dark:text-dark-text-secondary whitespace-pre-wrap'>
                {props.updateInfo.notes}
              </div>
            </div>
          </Show>

          {/* Error message */}
          <Show when={error()}>
            <div class='flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400'>
              <AlertCircle size={16} />
              {error()}
            </div>
          </Show>

          {/* Success message */}
          <Show when={success()}>
            <div class='flex items-center gap-2 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-600 dark:bg-emerald-900/20 dark:text-emerald-400'>
              <CheckCircle size={16} />
              更新已下载，应用将自动重启完成安装
            </div>
          </Show>

          {/* Actions */}
          <div class='flex justify-end gap-3 pt-2'>
            <button
              onClick={props.onClose}
              disabled={installing()}
              class={cn(
                'rounded-lg border border-gray-200 px-4 py-2 text-sm font-medium transition-colors',
                'text-gray-600 hover:bg-gray-50',
                'dark:border-dark-card-border dark:text-dark-text-secondary dark:hover:bg-dark-sidebar-border',
                'disabled:opacity-50 disabled:cursor-not-allowed',
              )}
            >
              {success() ? '关闭' : '稍后更新'}
            </button>
            <Show when={!success()}>
              <button
                onClick={handleInstall}
                disabled={installing()}
                class={cn(
                  'flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white transition-colors',
                  'hover:bg-primary-hover active:scale-[0.98]',
                  'disabled:opacity-50 disabled:cursor-not-allowed',
                )}
              >
                <Show
                  when={!installing()}
                  fallback={
                    <div class='h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent' />
                  }
                >
                  <Download size={16} />
                </Show>
                {installing() ? '下载中...' : '立即更新'}
              </button>
            </Show>
          </div>
        </div>
      </div>
    </div>
  )
}
