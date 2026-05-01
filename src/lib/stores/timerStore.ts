import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

export type TimerMode = "countup" | "pomodoro" | "countdown";
export type PomodoroPhase = "work" | "break";

interface TimerState {
    elapsed: number;
    remaining: number;
    initialRemaining: number;
    isRunning: boolean;
    timerMode: TimerMode;
    pomodoroPhase: PomodoroPhase;
    pomodoroWorkMinutes: number;
    pomodoroBreakMinutes: number;
    targetHours: number;
    targetMinutes: number;
    soundEnabled: boolean;
}

const initialState: TimerState = {
    elapsed: 0,
    remaining: 0,
    initialRemaining: 0,
    isRunning: false,
    timerMode: "countup",
    pomodoroPhase: "work",
    pomodoroWorkMinutes: 25,
    pomodoroBreakMinutes: 5,
    targetHours: 8,
    targetMinutes: 0,
    soundEnabled: true
};

function createTimerStore() {
    const { subscribe, set, update } = writable<TimerState>(initialState);

    let interval: ReturnType<typeof setInterval> | null = null;

    const stopInterval = () => {
        if (interval) {
            clearInterval(interval);
            interval = null;
        }
    };

    const startInterval = () => {
        stopInterval();
        interval = setInterval(() => {
            update(s => {
                const nextElapsed = s.elapsed + 1;
                let nextRemaining = s.remaining;
                let nextPhase = s.pomodoroPhase;
                let nextIsRunning = s.isRunning;

                if (s.timerMode !== "countup") {
                    if (s.remaining > 0) {
                        nextRemaining = s.remaining - 1;
                    }
                    if (nextRemaining <= 0) {
                        nextRemaining = 0;
                        if (s.timerMode === "pomodoro") {
                            nextPhase = s.pomodoroPhase === "work" ? "break" : "work";
                            const minutes = nextPhase === "work" ? s.pomodoroWorkMinutes : s.pomodoroBreakMinutes;
                            nextRemaining = minutes * 60;
                            // Sound notification should be handled in component
                        } else {
                            nextIsRunning = false;
                            stopInterval();
                        }
                    }
                }

                return {
                    ...s,
                    elapsed: nextElapsed,
                    remaining: nextRemaining,
                    pomodoroPhase: nextPhase,
                    isRunning: nextIsRunning
                };
            });
        }, 1000);
    };

    return {
        subscribe,
        start: () => {
            update(s => {
                if (s.isRunning) return s;
                startInterval();
                return { ...s, isRunning: true };
            });
        },
        pause: () => {
            update(s => {
                stopInterval();
                return { ...s, isRunning: false };
            });
        },
        toggle: () => {
            update(s => {
                if (s.isRunning) {
                    stopInterval();
                    return { ...s, isRunning: false };
                } else {
                    startInterval();
                    return { ...s, isRunning: true };
                }
            });
        },
        reset: () => {
            update(s => {
                stopInterval();
                let nextRemaining = 0;
                if (s.timerMode === "pomodoro") {
                    nextRemaining = s.pomodoroWorkMinutes * 60;
                } else if (s.timerMode === "countdown") {
                    nextRemaining = (s.targetHours * 60 + s.targetMinutes) * 60;
                }
                return {
                    ...s,
                    elapsed: 0,
                    remaining: nextRemaining,
                    initialRemaining: nextRemaining,
                    isRunning: false,
                    pomodoroPhase: "work"
                };
            });
        },
        setMode: (mode: TimerMode) => {
            update(s => {
                stopInterval();
                let nextRemaining = 0;
                if (mode === "pomodoro") {
                    nextRemaining = s.pomodoroWorkMinutes * 60;
                } else if (mode === "countdown") {
                    nextRemaining = (s.targetHours * 60 + s.targetMinutes) * 60;
                }
                return {
                    ...s,
                    timerMode: mode,
                    elapsed: 0,
                    remaining: nextRemaining,
                    initialRemaining: nextRemaining,
                    isRunning: false,
                    pomodoroPhase: "work"
                };
            });
        },
        updateSettings: (settings: Partial<TimerState>) => {
            update(s => {
                const newState = { ...s, ...settings };
                // If mode-defining settings changed, we might need to reset remaining
                if (settings.pomodoroWorkMinutes !== undefined && s.timerMode === 'pomodoro' && s.pomodoroPhase === 'work') {
                    newState.remaining = settings.pomodoroWorkMinutes * 60;
                    newState.initialRemaining = newState.remaining;
                }
                if (settings.targetHours !== undefined || settings.targetMinutes !== undefined) {
                    if (s.timerMode === 'countdown') {
                        newState.remaining = (newState.targetHours * 60 + newState.targetMinutes) * 60;
                        newState.initialRemaining = newState.remaining;
                    }
                }
                return newState;
            });
        }
    };
}

export const timerStore = createTimerStore();

export const formattedTime = derived(timerStore, ($timer) => {
    const totalSeconds = $timer.timerMode === "countup" ? $timer.elapsed : $timer.remaining;
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    const hStr = hours > 0 ? `${hours}:` : "";
    const mStr = minutes.toString().padStart(hours > 0 ? 2 : 1, "0");
    const sStr = seconds.toString().padStart(2, "0");

    return `${hStr}${mStr}:${sStr}`;
});

export const timerProgress = derived(timerStore, ($timer) => {
    if ($timer.timerMode === "countup") return 0;
    if ($timer.initialRemaining <= 0) return 0;
    return (1 - $timer.remaining / $timer.initialRemaining) * 100;
});
