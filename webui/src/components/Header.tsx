import { type Component } from 'solid-js'
import { Search, Plus, Menu } from 'lucide-solid'

interface HeaderProps {
  searchQuery: string
  totalAccounts: number
  onSearch: (query: string) => void
  onAddAccount: () => void
  onToggleSidebar: () => void
}

export const Header: Component<HeaderProps> = (props) => {
  return (
    <header class='flex items-center gap-4 border-b border-gray-100 px-6 py-3 bg-white dark:bg-dark-page-bg dark:border-dark-card-border'>
      {/* Mobile menu button */}
      <button
        onClick={props.onToggleSidebar}
        class='no-drag lg:hidden text-gray-500 dark:text-dark-text-secondary'
      >
        <Menu size={20} />
      </button>

      {/* Search bar */}
      <div class='no-drag relative flex-1'>
        <Search
          size={16}
          class='absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 dark:text-dark-text-secondary'
        />
        <input
          type='text'
          value={props.searchQuery}
          onInput={(e) => props.onSearch(e.currentTarget.value)}
          placeholder='搜索账号、标签或备注...'
          class='w-full rounded-lg border border-gray-100 bg-gray-50 py-2 pl-10 pr-4 text-sm outline-none transition-colors
            placeholder:text-gray-400 focus:border-primary focus:ring-1 focus:ring-primary/20
            dark:bg-dark-sidebar-bg dark:border-dark-card-border dark:text-dark-text dark:placeholder:text-dark-text-secondary'
        />
      </div>

      {/* Add account button */}
      <button
        onClick={props.onAddAccount}
        class='no-drag flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover active:scale-[0.98]'
      >
        <Plus size={16} />
        <span class='hidden sm:inline'>添加账号</span>
      </button>
    </header>
  )
}
