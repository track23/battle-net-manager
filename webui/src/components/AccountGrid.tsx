import { For, Show, type Component } from "solid-js";
import type { Account, Group } from "../types";
import { AccountCard } from "./AccountCard";
import { Sparkles } from "lucide-solid";
import { useI18n } from '../i18n'

interface AccountGridProps {
  accounts: Account[];
  groups: Group[];
  activeAccountId: string | null;
  switchingAccountId: string | null;
  onSwitch: (id: string) => void;
  onEdit: (account: Account) => void;
  onDelete: (id: string) => void;
  onMoveGroup: (accountId: string, groupId: string) => void;
}

export const AccountGrid: Component<AccountGridProps> = (props) => {
  const { t } = useI18n()
  return (
    <div class="flex-1 overflow-y-auto p-6 hide-scrollbar">
      <Show
        when={props.accounts.length > 0}
        fallback={
          <div class="flex flex-col items-center justify-center py-20 text-center">
            <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-gray-100 dark:bg-dark-sidebar-bg">
              <Sparkles
                size={28}
                class="text-gray-400 dark:text-dark-text-secondary"
              />
            </div>
            <p class="text-sm font-medium text-gray-500 dark:text-dark-text-secondary">
              {t('noAccounts')}
            </p>
            <p class="mt-1 text-xs text-gray-400 dark:text-dark-text-secondary">
              {t('noAccountsHint')}
            </p>
          </div>
        }
      >
        <div class="grid gap-4 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3">
          <For each={props.accounts}>
            {(account) => (
              <AccountCard
                account={account}
                groups={props.groups}
                isActive={props.activeAccountId === account.Id}
                isSwitching={props.switchingAccountId === account.Id}
                onSwitch={props.onSwitch}
                onEdit={props.onEdit}
                onDelete={props.onDelete}
                onMoveGroup={props.onMoveGroup}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
