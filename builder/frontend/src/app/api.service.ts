import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

export interface Target {
  triple: string;
  label: string;
  artifact: string;
  needs_cross: boolean;
  is_host: boolean;
}

export interface ConfigInfo {
  name: string;
  components: string[];
  size: number;
}

export interface ConfigDetail {
  name: string;
  content: string;
  components: string[];
}

export interface ValidationResult {
  ok: boolean;
  output: string;
}

export interface BuildRecord {
  id: number;
  config: string;
  version: string;
  target: string;
  components: string[];
  status: string;
  size: number;
  created_at: number;
  artifact: string | null;
  log_file: string | null;
}

export interface Versions {
  latest: string | null;
  versions: string[];
}

/** Thin client over the builder REST API (served same-origin under /api). */
@Injectable({ providedIn: 'root' })
export class ApiService {
  private http = inject(HttpClient);

  targets() {
    return firstValueFrom(this.http.get<Target[]>('/api/targets'));
  }

  versions() {
    return firstValueFrom(this.http.get<Versions>('/api/versions'));
  }

  listConfigs() {
    return firstValueFrom(this.http.get<ConfigInfo[]>('/api/configs'));
  }

  getConfig(name: string) {
    return firstValueFrom(this.http.get<ConfigDetail>(`/api/configs/${encodeURIComponent(name)}`));
  }

  createConfig(name: string, content = '') {
    return firstValueFrom(this.http.post('/api/configs', { name, content }));
  }

  saveConfig(name: string, content: string) {
    return firstValueFrom(
      this.http.put(`/api/configs/${encodeURIComponent(name)}`, { content }),
    );
  }

  deleteConfig(name: string) {
    return firstValueFrom(this.http.delete(`/api/configs/${encodeURIComponent(name)}`));
  }

  duplicateConfig(name: string, to: string) {
    return firstValueFrom(
      this.http.post(`/api/configs/${encodeURIComponent(name)}/duplicate`, { to }),
    );
  }

  renameConfig(name: string, to: string) {
    return firstValueFrom(
      this.http.post(`/api/configs/${encodeURIComponent(name)}/rename`, { to }),
    );
  }

  validate(name: string, ref: string | null) {
    const q = ref ? `?ref=${encodeURIComponent(ref)}` : '';
    return firstValueFrom(
      this.http.post<ValidationResult>(
        `/api/configs/${encodeURIComponent(name)}/validate${q}`,
        {},
      ),
    );
  }

  startBuild(name: string, target: string | null, ref: string | null) {
    return firstValueFrom(
      this.http.post<{ build_id: number }>(`/api/configs/${encodeURIComponent(name)}/build`, {
        target,
        ref,
      }),
    );
  }

  listBuilds() {
    return firstValueFrom(this.http.get<BuildRecord[]>('/api/builds'));
  }

  /** URL to download a finished build's artifact. */
  artifactUrl(id: number) {
    return `/api/builds/${id}/artifact`;
  }

  /** Open a WebSocket that streams a build's logs. */
  openLogSocket(id: number): WebSocket {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    return new WebSocket(`${proto}://${location.host}/api/builds/${id}/logs`);
  }
}
