import { create } from "zustand";
import type { Job, FlashProgress } from "../types";

interface JobStore {
  jobs: Job[];
  activeFlash: FlashProgress | null;
  setJobs: (jobs: Job[]) => void;
  addJob: (job: Job) => void;
  updateJob: (id: string, patch: Partial<Job>) => void;
  setActiveFlash: (progress: FlashProgress | null) => void;
}

export const useJobStore = create<JobStore>((set) => ({
  jobs: [],
  activeFlash: null,
  setJobs: (jobs) => set({ jobs }),
  addJob: (job) => set((s) => ({ jobs: [...s.jobs, job] })),
  updateJob: (id, patch) =>
    set((s) => ({
      jobs: s.jobs.map((j) => (j.id === id ? { ...j, ...patch } : j)),
    })),
  setActiveFlash: (progress) => set({ activeFlash: progress }),
}));
