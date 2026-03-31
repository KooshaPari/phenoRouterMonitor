import { create } from 'zustand';
import type { WorkPackage } from '../types';

// ============================================================================
// AgilePlus State Store
// Centralized client state management using Zustand
// ============================================================================

interface AgilePlusState {
  // Work Package Data
  workPackages: WorkPackage[];
  selectedWP: string | null;
  loading: boolean;

  // Filters
  filters: {
    status: string;
    assignee: string;
    priority: string;
  };

  // Actions
  setWorkPackages: (wps: WorkPackage[]) => void;
  selectWorkPackage: (id: string | null) => void;
  setLoading: (loading: boolean) => void;
  updateFilters: (filters: Partial<AgilePlusState['filters']>) => void;
  clearFilters: () => void;
}

/**
 * Main AgilePlus store
 * Manages work packages, filters, and UI state across pages
 */
export const useAgilePlusStore = create<AgilePlusState>((set) => ({
  workPackages: [],
  selectedWP: null,
  loading: false,
  filters: {
    status: 'all',
    assignee: '',
    priority: '',
  },

  setWorkPackages: (workPackages) => set({ workPackages }),

  selectWorkPackage: (id) => set({ selectedWP: id }),

  setLoading: (loading) => set({ loading }),

  updateFilters: (newFilters) =>
    set((state) => ({
      filters: { ...state.filters, ...newFilters },
    })),

  clearFilters: () =>
    set({
      filters: {
        status: 'all',
        assignee: '',
        priority: '',
      },
    }),
}));

export default useAgilePlusStore;
