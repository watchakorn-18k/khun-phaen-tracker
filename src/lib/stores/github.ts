import { writable } from "svelte/store";
import { browser } from "$app/environment";

const REPO = "fakduai-logistics-and-digital-platform/khun-phaen-tracker";
const CACHE_KEY = "github_stars_cache";
const CACHE_DURATION = 1000 * 60 * 5; // 5 minutes

interface GitHubData {
  stars: string;
  lastFetched: number;
}

function createGitHubStore() {
  const initialData: GitHubData = {
    stars: "...",
    lastFetched: 0,
  };

  const { subscribe, set, update } = writable<GitHubData>(initialData);

  if (browser) {
    const cached = localStorage.getItem(CACHE_KEY);
    if (cached) {
      try {
        const data = JSON.parse(cached);
        if (Date.now() - data.lastFetched < CACHE_DURATION) {
          set(data);
        } else {
          // Update stars if cache is still present but stale
          set({ ...data, stars: data.stars });
        }
      } catch (e) {
        console.error("Failed to parse cached stars:", e);
      }
    }
  }

  async function fetchStars() {
    try {
      const res = await fetch(`https://api.github.com/repos/${REPO}`);
      if (!res.ok) throw new Error(`GitHub API error: ${res.status}`);

      const data = await res.json();
      const stars =
        data.stargazers_count !== undefined
          ? data.stargazers_count.toLocaleString()
          : "0";

      const newData = {
        stars: stars,
        lastFetched: Date.now(),
      };

      set(newData);
      if (browser) {
        localStorage.setItem(CACHE_KEY, JSON.stringify(newData));
      }
    } catch (e) {
      console.error("Failed to fetch github stars:", e);
      // Reuse old value on error if possible
      update((current) => ({ ...current }));
    }
  }

  return {
    subscribe,
    fetchStars,
    updateCache: (stars: string) => {
      const newData = { stars, lastFetched: Date.now() };
      set(newData);
      if (browser) localStorage.setItem(CACHE_KEY, JSON.stringify(newData));
    },
  };
}

export const githubStore = createGitHubStore();
