import { useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

import { useVisualisation } from './useVisualisation.ts';
import { useAppStatusStore } from '@app/store/useAppStatusStore.ts';
import useAppStateStore from '@app/store/appStateStore.ts';
import { useCPUStatusStore } from '@app/store/useCPUStatusStore.ts';
import { useUIStore } from '@app/store/useUIStore.ts';

export enum MiningButtonStateText {
    STARTING = 'Starting mining',
    STARTED = 'Pause mining',
    CONNECTION_LOST = 'Cancel mining',
    CHANGING_MODE = 'Changing mode',
    START = 'Start mining',
    AUTO_MINING = 'Waiting for idle',
    AUTO_MINING_STARTED = 'Started auto mining',
}

export function useMiningControls() {
    const handleVisual = useVisualisation();
    const progress = useAppStateStore((s) => s.setupProgress);
    const isMining = useCPUStatusStore((s) => s.is_mining);
    const isAutoMining = useAppStatusStore((s) => s.auto_mining);
    const hashRate = useCPUStatusStore((s) => s.hash_rate);
    const { isMiningEnabled, setIsMiningEnabled } = useUIStore((s) => ({
        isMiningEnabled: s.isMiningEnabled,
        setIsMiningEnabled: s.setIsMiningEnabled,
    }));
    const { isConnectionLostDuringMining } = useUIStore((s) => ({
        isConnectionLostDuringMining: s.isConnectionLostDuringMining,
        setIsConnectionLostDuringMining: s.setIsConnectionLostDuringMining,
    }));

    const { isChangingMode, setIsChangingMode } = useUIStore((s) => ({
        isChangingMode: s.isChangingMode,
        setIsChangingMode: s.setIsChangingMode,
    }));
    const { isMiningInProgress } = useUIStore((s) => ({
        isMiningInProgress: s.isMiningInProgress,
        setIsMiningInProgress: s.setIsMiningInProgress,
    }));

    const isLoading = useMemo(() => {
        if (isConnectionLostDuringMining) return false;
        if (isChangingMode) return true;
        return !isMining && isMiningEnabled;
    }, [isMining, isMiningEnabled, isConnectionLostDuringMining, isChangingMode]);

    const isWaitingForHashRate = useMemo(() => {
        return isLoading || (isMining && hashRate <= 0);
    }, [isMining, hashRate, isLoading]);

    const shouldMiningControlsBeEnabled = useMemo(() => {
        if (isConnectionLostDuringMining) return true;

        if (isChangingMode) return false;

        if (!isMining && isMiningEnabled) return false;

        if (isMining && progress < 1) return true;

        if (progress >= 1 && !isAutoMining) return true;
        return false;
    }, [
        isAutoMining,
        isWaitingForHashRate,
        isMining,
        progress,
        isMiningEnabled,
        isConnectionLostDuringMining,
        isChangingMode,
    ]);

    const shouldAutoMiningControlsBeEnabled = useMemo(() => {
        if (isMiningEnabled && !isAutoMining) return false;

        if (isChangingMode) return false;

        if (isMining && progress < 1) return true;
        if (progress >= 1) return true;
        return false;
    }, [isAutoMining, isMining, progress, isMiningEnabled, isChangingMode]);

    const startMining = useCallback(async () => {
        setIsMiningEnabled(true);
        await invoke('start_mining', {}).catch(() => {
            setIsMiningEnabled(false);
        });
    }, []);

    const stopMining = useCallback(async () => {
        setIsMiningEnabled(false);
        await invoke('stop_mining', {})
            .then(async () => {
                handleVisual('stop');
            })
            .catch(() => {
                setIsMiningEnabled(true);
            });
    }, [handleVisual]);

    const cancelMining = useCallback(async () => {
        setIsMiningEnabled(false);
        await invoke('stop_mining', {}).then(async () => {
            handleVisual('start');
            handleVisual('stop');
        });
    }, []);

    const changeMode = useCallback(
        async (mode: string) => {
            const hasBeenMining = isMiningInProgress;

            if (!hasBeenMining || isAutoMining) {
                await invoke('set_mode', { mode });
                return;
            }

            setIsChangingMode(true);
            if (hasBeenMining && !isConnectionLostDuringMining) {
                await stopMining();
            }

            if (isConnectionLostDuringMining) {
                await cancelMining();
            }

            await invoke('set_mode', { mode });

            if (hasBeenMining && !isConnectionLostDuringMining) {
                setTimeout(async () => {
                    await startMining();
                }, 2000);
            }

            if (isConnectionLostDuringMining) {
                setIsChangingMode(false);
            }
        },
        [isMiningInProgress, isConnectionLostDuringMining, isAutoMining]
    );

    const getMiningButtonStateText = useCallback(() => {
        if (isChangingMode) {
            return MiningButtonStateText.CHANGING_MODE;
        }

        if (isConnectionLostDuringMining) {
            return MiningButtonStateText.CONNECTION_LOST;
        }

        if (isMiningEnabled && !isMining) {
            return MiningButtonStateText.STARTING;
        }

        if (isAutoMining && isMining) {
            return MiningButtonStateText.AUTO_MINING_STARTED;
        }

        if (isAutoMining) {
            return MiningButtonStateText.AUTO_MINING;
        }

        if (isMining) {
            return MiningButtonStateText.STARTED;
        }

        return MiningButtonStateText.START;
    }, [isAutoMining, isMining, isWaitingForHashRate, isMiningEnabled, isConnectionLostDuringMining, isChangingMode]);

    return {
        cancelMining,
        changeMode,
        isConnectionLostDuringMining,
        isLoading,
        startMining,
        stopMining,
        getMiningButtonStateText,
        isWaitingForHashRate,
        shouldMiningControlsBeEnabled,
        shouldAutoMiningControlsBeEnabled,
    };
}
