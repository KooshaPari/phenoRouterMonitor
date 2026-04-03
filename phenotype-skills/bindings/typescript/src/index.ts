/**
 * TypeScript bindings for Phenotype Skills
 */

import * as ffi from 'ffi-napi';
import * as ref from 'ref-napi';

/**
 * Execution modes for skill sandboxing
 */
export enum ExecutionMode {
  InProcess = 'in_process',
  WASM = 'wasm',
  GVisor = 'gvisor',
  Firecracker = 'firecracker',
}

/**
 * Skill manifest definition
 */
export interface SkillManifest {
  name: string;
  version: string;
  description: string;
  author: string;
  license: string;
  entryPoint: string;
  dependencies: SkillDependency[];
  metadata: Record<string, string>;
}

/**
 * Skill dependency definition
 */
export interface SkillDependency {
  name: string;
  versionReq: string;
  optional: boolean;
}

/**
 * Skill registry for managing skills
 */
export class SkillRegistry {
  private lib: any;

  constructor() {
    // Load the native library
    const libPath = this.getLibraryPath();
    this.lib = ffi.Library(libPath, {
      'skill_registry_new': ['pointer', []],
      'skill_registry_free': ['void', ['pointer']],
      'skill_registry_register': ['int', ['pointer', 'string']],
      'skill_registry_unregister': ['int', ['pointer', 'string']],
      'skill_registry_list': ['int', ['pointer', 'pointer', 'int']],
    });
  }

  private getLibraryPath(): string {
    const platform = process.platform;
    switch (platform) {
      case 'darwin':
        return '../../target/release/libphenotype_skills.dylib';
      case 'linux':
        return '../../target/release/libphenotype_skills.so';
      case 'win32':
        return '../../target/release/phenotype_skills.dll';
      default:
        throw new Error(`Unsupported platform: ${platform}`);
    }
  }

  /**
   * Register a skill from a manifest file
   */
  register(manifestPath: string): void {
    const handle = this.lib.skill_registry_new();
    try {
      const result = this.lib.skill_registry_register(handle, manifestPath);
      if (result !== 0) {
        throw new Error(`Failed to register skill: ${result}`);
      }
    } finally {
      this.lib.skill_registry_free(handle);
    }
  }

  /**
   * Unregister a skill by ID
   */
  unregister(skillId: string): void {
    const handle = this.lib.skill_registry_new();
    try {
      const result = this.lib.skill_registry_unregister(handle, skillId);
      if (result !== 0) {
        throw new Error(`Failed to unregister skill: ${result}`);
      }
    } finally {
      this.lib.skill_registry_free(handle);
    }
  }
}

export { SkillRegistry as default };
