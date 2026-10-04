import { useTranslation } from 'react-i18next'
import type { AppSettings } from '../../../shared'
import { Input } from '../ui/input'
import { SectionTitle, SettingRow, Switch } from './SettingsPrimitives'

interface DisplaySettingsSectionProps {
  settings: AppSettings
  onUpdate: (patch: Partial<AppSettings>) => void
  disabled: boolean
}

export function DisplaySettingsSection({
  settings,
  onUpdate,
  disabled
}: DisplaySettingsSectionProps): React.JSX.Element {
  const { t } = useTranslation()
  return (
    <section>
      <SectionTitle>{t('settings:display_title')}</SectionTitle>
      <div className="bg-card border border-border rounded-lg divide-y divide-border">
        <div className="px-4">
          <SettingRow
            label={t('settings:fallback_display')}
            description={t('settings:fallback_display_desc')}
          >
            <Switch
              checked={settings.fallbackDisplay}
              onCheckedChange={(v) => onUpdate({ fallbackDisplay: v })}
              disabled={disabled}
            />
          </SettingRow>
        </div>
        <div className="px-4">
          <SettingRow
            label={t('settings:keep_screen_on')}
            description={t('settings:keep_screen_on_desc')}
          >
            <Switch
              checked={settings.keepScreenOn}
              onCheckedChange={(v) => onUpdate({ keepScreenOn: v })}
              disabled={disabled}
            />
          </SettingRow>
        </div>
        <div className="px-4">
          <SettingRow
            label={t('settings:auto_add_display_on_launch')}
            description={t('settings:auto_add_display_on_launch_desc')}
          >
            <Switch
              checked={settings.autoAddDisplayOnLaunch}
              onCheckedChange={(v) => onUpdate({ autoAddDisplayOnLaunch: v })}
              disabled={disabled}
            />
          </SettingRow>
        </div>
        {settings.autoAddDisplayOnLaunch && (
          <div className="px-4">
            <SettingRow
              label={t('settings:auto_add_display_delay')}
              description={t('settings:auto_add_display_delay_desc')}
            >
              <Input
                type="number"
                min={0}
                max={600}
                className="w-24"
                value={String(settings.autoAddDisplayDelaySecs)}
                disabled={disabled}
                onChange={(e) => {
                  const parsed = Number.parseInt(e.target.value, 10)
                  if (Number.isNaN(parsed)) return
                  onUpdate({
                    autoAddDisplayDelaySecs: Math.max(0, Math.min(600, parsed))
                  })
                }}
              />
            </SettingRow>
          </div>
        )}
        <div className="px-4">
          <SettingRow
            label={t('settings:auto_switch_to_virtual_on_launch')}
            description={t('settings:auto_switch_to_virtual_on_launch_desc')}
          >
            <Switch
              checked={settings.autoSwitchToVirtualOnLaunch}
              onCheckedChange={(v) => onUpdate({ autoSwitchToVirtualOnLaunch: v })}
              disabled={disabled}
            />
          </SettingRow>
        </div>
      </div>
    </section>
  )
}
