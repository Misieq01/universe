import { useCallback, useEffect, useMemo, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

import { useVisualisation } from './useVisualisation.ts';
import { useAppStatusStore } from '@app/store/useAppStatusStore.ts';
import useAppStateStore from '@app/store/appStateStore.ts';
import { useCPUStatusStore } from '@app/store/useCPUStatusStore.ts';
import { useUIStore } from '@app/store/useUIStore.ts';

export enum MiningButtonStateText {
    STARTING = 'starting-mining',
    STARTED = 'start-mining',
    CONNECTION_LOST = 'cancel-mining',
    START = 'start-mining',
    AUTO_MINING = 'waiting-for-idle',
    AUTO_MINING_STARTED = 'started-auto-mining',
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
    const { isConnectionLostDuringMining, setIsConnectionLostDuringMining } = useUIStore((s) => ({
        isConnectionLostDuringMining: s.isConnectionLostDuringMining,
        setIsConnectionLostDuringMining: s.setIsConnectionLostDuringMining,
    }));

    const isMiningInProgress = useRef(false);

    const isLoading = useMemo(() => {
        if (isConnectionLostDuringMining) return false;
        return !isMining && isMiningEnabled;
    }, [isMining, isMiningEnabled, isConnectionLostDuringMining]);

    const isWaitingForHashRate = useMemo(() => {
        return isLoading || (isMining && hashRate <= 0);
    }, [isMining, hashRate, isLoading]);

    const shouldMiningControlsBeEnabled = useMemo(() => {
        if (isConnectionLostDuringMining) return true;

        if (!isMining && isMiningEnabled) return false;

        if (isMining && progress < 1) return true;

        return progress >= 1 && !isAutoMining;
    }, [isAutoMining, isMining, progress, isMiningEnabled, isConnectionLostDuringMining]);

    const shouldAutoMiningControlsBeEnabled = useMemo(() => {
        if (isMiningEnabled && !isAutoMining) return false;
        if (isMining && progress < 1) return true;
        return progress >= 1;
    }, [isAutoMining, isMining, progress, isMiningEnabled]);

    const startMining = useCallback(async () => {
        setIsMiningEnabled(true);
        await invoke('start_mining', {})
            .then(() => {
                console.info(`mining started`);
            })
            .catch(() => {
                setIsMiningEnabled(false);
            });
    }, [setIsMiningEnabled]);

    const stopMining = useCallback(async () => {
        setIsMiningEnabled(false);
        await invoke('stop_mining', {})
            .then(async () => {
                console.info(`mining stopped`);
                await handleVisual('stop');
            })
            .catch(() => {
                setIsMiningEnabled(true);
            });
    }, [handleVisual, setIsMiningEnabled]);

    const cancelMining = useCallback(async () => {
        setIsMiningEnabled(false);
        await invoke('stop_mining', {}).then(async () => {
            console.info(`mining canceled`);
            await handleVisual('start');
            await handleVisual('stop');
        });
    }, [handleVisual, setIsMiningEnabled]);

    useEffect(() => {
        if (isMining && isMiningEnabled) {
            if (isConnectionLostDuringMining) setIsConnectionLostDuringMining(false);
            console.info('useEffect: handleVisual start');
            handleVisual('start').then(() => {
                isMiningInProgress.current = true;
            });
        }

        if (!isMining && !isMiningEnabled) {
            if (isConnectionLostDuringMining) setIsConnectionLostDuringMining(false);
            console.info('useEffect: handleVisual stop');
            handleVisual('stop').then(() => {
                isMiningInProgress.current = false;
            });
        }

        if (!isMining && isMiningInProgress.current) {
            console.info('useEffect: handleVisual pause');
            setIsConnectionLostDuringMining(true);
            void handleVisual('pause');
        }
    }, [handleVisual, isMining, isMiningEnabled, isConnectionLostDuringMining, setIsConnectionLostDuringMining]);

    const getMiningButtonStateText = useCallback(() => {
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
    }, [isAutoMining, isMining, isMiningEnabled, isConnectionLostDuringMining]);

    return {
        cancelMining,
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
