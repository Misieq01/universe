import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useMiningStore } from '@app/store/useMiningStore';
import { useMiningMetricsStore } from '@app/store/useMiningMetricsStore';
import { GpuStatus } from '@app/types/app-status';

export interface DetectedAvailableGpuEnginesPayload {
    engines: string[];
    selected_engine: string;
}

export interface DetectedGpuHardwarePayload {
    devices: GpuStatus[];
}

export const useListenForGpuEngines = () => {
    const setAvailableEngines = useMiningStore((state) => state.setAvailableEngines);
    const setGpus = useMiningMetricsStore((state) => state.setGpuHardware);

    useEffect(() => {
        const listenerForDetectecAvailableGpuEngines = listen(
            'detected-available-gpu-engines',
            ({ payload }: { payload: DetectedAvailableGpuEnginesPayload }) => {
                setAvailableEngines(payload.engines, payload.selected_engine);
            }
        );

        return () => {
            listenerForDetectecAvailableGpuEngines.then((unlisten) => unlisten());
        };
    }, [setAvailableEngines]);

    useEffect(() => {
        const listenerForGpuHardware = listen(
            'detected-devices',
            ({ payload }: { payload: DetectedGpuHardwarePayload }) => {
                setGpus(payload.devices);
            }
        );

        return () => {
            listenerForGpuHardware.then((unlisten) => unlisten());
        };
    }, []);
};
