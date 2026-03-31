import { useEffect } from 'react';
import axios from 'axios';
import { useAgilePlusStore } from '../stores/agileplus';
import type { WorkPackage } from '../types';

// ============================================================================
// useWorkPackages Hook
// Fetch and manage work package data from API
// ============================================================================

interface UseWorkPackagesOptions {
  skip?: boolean;
}

/**
 * Hook to fetch and manage work packages
 * Integrates with Zustand store and API
 *
 * @example
 * const { workPackages, loading, error } = useWorkPackages();
 */
export function useWorkPackages(options: UseWorkPackagesOptions = {}) {
  const { skip = false } = options;
  const { workPackages, setWorkPackages, setLoading } = useAgilePlusStore();

  useEffect(() => {
    if (skip) return;

    const fetchWorkPackages = async () => {
      setLoading(true);
      try {
        const response = await axios.get<WorkPackage[]>('/api/work-packages');
        setWorkPackages(response.data);
      } catch (error) {
        console.error('Failed to fetch work packages:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchWorkPackages();
  }, [skip, setWorkPackages, setLoading]);

  return {
    workPackages,
    loading: useAgilePlusStore((state) => state.loading),
  };
}

export default useWorkPackages;
