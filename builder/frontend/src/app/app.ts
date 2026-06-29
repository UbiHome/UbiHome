import { Component, OnInit, computed, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import {
  ApiService,
  BuildRecord,
  ConfigInfo,
  Target,
  ValidationResult,
} from './api.service';

/** Base-config keys that are NOT platform components.
 *  Mirror of `RESERVED` in engine/src/platforms.rs (BaseConfig fields). */
const RESERVED = new Set([
  'ubihome',
  'logger',
  'button',
  'sensor',
  'binary_sensor',
  'number',
  'switch',
  'light',
  'text_sensor',
]);

/** Client-side mirror of the engine's component detection, for a live panel. */
function detectPlatforms(config: string): string[] {
  const found: string[] = [];
  for (const line of config.split('\n')) {
    if (
      line.startsWith(' ') ||
      line.startsWith('\t') ||
      line === '' ||
      line.startsWith('#') ||
      line.startsWith('-')
    ) {
      continue;
    }
    const key = line.split(':')[0]?.trim();
    if (!key || RESERVED.has(key)) continue;
    if (!found.includes(key)) found.push(key);
  }
  return found.sort();
}

@Component({
  selector: 'app-root',
  imports: [FormsModule],
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App implements OnInit {
  private api = inject(ApiService);

  view = signal<'editor' | 'history'>('editor');

  configs = signal<ConfigInfo[]>([]);
  targets = signal<Target[]>([]);
  versions = signal<string[]>([]);
  builds = signal<BuildRecord[]>([]);

  selected = signal<string | null>(null);
  content = signal('');
  selectedTarget = signal<string>(''); // '' == native host
  selectedVersion = signal<string>(''); // '' == latest stable

  validation = signal<ValidationResult | null>(null);
  validating = signal(false);

  building = signal(false);
  logLines = signal<string[]>([]);
  banner = signal<string | null>(null);

  /** Live component detection from the editor content. */
  detected = computed(() => detectPlatforms(this.content()));

  async ngOnInit() {
    await this.refreshConfigs();
    try {
      this.targets.set(await this.api.targets());
    } catch (e) {
      this.flash('Could not load targets: ' + e);
    }
    // Versions require a clone/fetch; load in the background.
    this.api
      .versions()
      .then((v) => this.versions.set(v.versions))
      .catch((e) => this.flash('Could not load versions: ' + e));
  }

  private flash(msg: string) {
    this.banner.set(msg);
    setTimeout(() => this.banner.set(null), 4000);
  }

  async refreshConfigs() {
    this.configs.set(await this.api.listConfigs());
  }

  async openConfig(name: string) {
    const detail = await this.api.getConfig(name);
    this.selected.set(name);
    this.content.set(detail.content);
    this.validation.set(null);
    this.logLines.set([]);
    this.view.set('editor');
  }

  async newConfig() {
    const name = prompt('New config file name (e.g. living-room.yml)');
    if (!name) return;
    try {
      await this.api.createConfig(name);
      await this.refreshConfigs();
      await this.openConfig(name);
    } catch (e: any) {
      this.flash(e?.error?.error ?? 'Could not create config');
    }
  }

  async save() {
    const name = this.selected();
    if (!name) return;
    await this.api.saveConfig(name, this.content());
    await this.refreshConfigs();
    this.flash('Saved');
  }

  async deleteConfig(name: string, ev: Event) {
    ev.stopPropagation();
    if (!confirm(`Delete ${name}?`)) return;
    await this.api.deleteConfig(name);
    if (this.selected() === name) {
      this.selected.set(null);
      this.content.set('');
    }
    await this.refreshConfigs();
  }

  async duplicate(name: string, ev: Event) {
    ev.stopPropagation();
    const to = prompt('Duplicate as:', name.replace(/\.ya?ml$/, '') + '-copy.yml');
    if (!to) return;
    await this.api.duplicateConfig(name, to);
    await this.refreshConfigs();
  }

  async validate() {
    const name = this.selected();
    if (!name) return;
    await this.save();
    this.validating.set(true);
    this.validation.set(null);
    try {
      this.validation.set(await this.api.validate(name, this.selectedVersion() || null));
    } catch (e: any) {
      this.validation.set({ ok: false, output: e?.error?.error ?? String(e) });
    } finally {
      this.validating.set(false);
    }
  }

  async build() {
    const name = this.selected();
    if (!name) return;
    await this.save();
    this.building.set(true);
    this.logLines.set([]);
    try {
      const { build_id } = await this.api.startBuild(
        name,
        this.selectedTarget() || null,
        this.selectedVersion() || null,
      );
      const ws = this.api.openLogSocket(build_id);
      ws.onmessage = (ev) => {
        this.logLines.update((lines) => [...lines, ev.data as string]);
      };
      ws.onclose = async () => {
        this.building.set(false);
        await this.refreshBuilds();
      };
      ws.onerror = () => {
        this.logLines.update((l) => [...l, '[log stream error]']);
        this.building.set(false);
      };
    } catch (e: any) {
      this.logLines.update((l) => [...l, 'ERROR: ' + (e?.error?.error ?? String(e))]);
      this.building.set(false);
    }
  }

  async refreshBuilds() {
    this.builds.set(await this.api.listBuilds());
  }

  async showHistory() {
    await this.refreshBuilds();
    this.view.set('history');
  }

  artifactUrl(id: number) {
    return this.api.artifactUrl(id);
  }

  fmtSize(bytes: number) {
    return bytes > 0 ? (bytes / 1_048_576).toFixed(1) + ' MB' : '—';
  }

  fmtTime(secs: number) {
    return secs ? new Date(secs * 1000).toLocaleString() : '';
  }
}
