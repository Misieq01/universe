import { Typography } from '@app/components/elements/Typography';
import { useMiningStore } from '@app/store/useMiningStore';
import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { GpuThreads, MaxConsumptionLevels } from '@app/types/app-status';
import { RangeInputComponent } from './RangeInput';
import { useAppConfigStore } from '@app/store/useAppConfigStore';
import {
    CustomLevelsHeader,
    CustomLevelsContent,
    SuccessContainer,
    TopRightContainer,
} from './CustomPowerLevelsDialog.styles.ts';
import { useTranslation } from 'react-i18next';
import { Divider } from '@app/components/elements/Divider.tsx';
import { IconButton } from '@app/components/elements/buttons/IconButton.tsx';
import { IoClose } from 'react-icons/io5';
import { LinearProgress } from '@app/components/elements/LinearProgress.tsx';
import { Controller, useFieldArray, useForm } from 'react-hook-form';
import { modeType } from '@app/store/types.ts';
import { Button } from '@app/components/elements/buttons/Button.tsx';

// took from gpu_miner_adapter.rs
const ECO_MODE_GPU_GRID_SIZE = 2;
const LUDICROUS_MODE_GPU_GRID_SIZE = 900;

enum FormFields {
    CPU = 'cpu',
    GPUS = 'gpus',
}

interface FormValues {
    [FormFields.CPU]: number;
    [FormFields.GPUS]: GpuThreads[];
}

const resolveCpuInitialThreads = (configCpuLevels: number | undefined, mode: modeType | undefined) => {
    switch (mode) {
        case 'Eco':
            return configCpuLevels || ECO_MODE_GPU_GRID_SIZE;
        case 'Ludicrous':
            return configCpuLevels || LUDICROUS_MODE_GPU_GRID_SIZE;
        default:
            return 0;
    }
};

const resolveGpuInitialThreads = (
    configGpuLevels: GpuThreads[] | undefined,
    mode: modeType | undefined,
    maxAvailableThreads: MaxConsumptionLevels
) => {
    if (configGpuLevels && configGpuLevels.length > 0) {
        return configGpuLevels;
    } else {
        switch (mode) {
            case 'Eco':
                return maxAvailableThreads.max_gpus_threads.map((gpu) => ({
                    gpu_name: gpu.gpu_name,
                    max_gpu_threads: ECO_MODE_GPU_GRID_SIZE,
                }));
            case 'Ludicrous':
                return maxAvailableThreads.max_gpus_threads.map((gpu) => ({
                    gpu_name: gpu.gpu_name,
                    max_gpu_threads: LUDICROUS_MODE_GPU_GRID_SIZE,
                }));
            default:
                return [];
        }
    }
};

export function CustomPowerLevelsDialog({
    maxAvailableThreads,
    handleClose,
}: {
    maxAvailableThreads: MaxConsumptionLevels;
    handleClose: () => void;
}) {
    const { t } = useTranslation('settings', { useSuspense: false });
    const [saved, setSaved] = useState(false);

    const mode = useAppConfigStore((s) => s.mode);
    const configCpuLevels = useAppConfigStore((s) => s.custom_max_cpu_usage);
    const configGpuLevels = useAppConfigStore((s) => s.custom_max_gpu_usage);

    const changeMiningMode = useMiningStore((s) => s.changeMiningMode);
    const isChangingMode = useMiningStore((s) => s.isChangingMode);

    const { control, handleSubmit, setValue, getValues } = useForm<FormValues>({
        defaultValues: {
            [FormFields.CPU]: resolveCpuInitialThreads(configCpuLevels, mode),
            [FormFields.GPUS]: resolveGpuInitialThreads(configGpuLevels, mode, maxAvailableThreads),
        },
    });

    const { fields } = useFieldArray({
        control,
        name: FormFields.GPUS,
        keyName: 'id',
    });

    useEffect(() => {
        // Remove save animation
        if (saved) {
            const timeout = setTimeout(() => setSaved(false), 3000);
            return () => clearTimeout(timeout);
        }
    }, [saved]);

    const onSubmit = useCallback(
        (data: FormValues) => {
            changeMiningMode({
                mode: 'Custom',
                customCpuLevels: data[FormFields.CPU],
                customGpuLevels: data[FormFields.GPUS],
            }).then(() => setSaved(true));
        },
        [changeMiningMode]
    );

    console.log('values', getValues());
    console.log('max values', maxAvailableThreads);

    // const gpuValue = useMemo(() => {
    //     if (mode !== 'Custom' && maxLevels.max_gpu_available) {
    //         return mode === 'Eco' ? 3 : Math.min(maxLevels.max_gpu_available, 800);
    //     }
    //     return configGpuLevels || 0;
    // }, [mode, maxLevels.max_gpu_available, configGpuLevels]);

    // const cpuValue = useMemo(() => {
    //     if (mode !== 'Custom' && maxLevels.max_cpu_available) {
    //         return mode === 'Eco' ? Math.round(maxLevels.max_cpu_available * 0.3) : maxLevels.max_cpu_available;
    //     }
    //     return configCpuLevels || 0;
    // }, [mode, maxLevels.max_cpu_available, configCpuLevels]);

    // const handleChangeCpu = useCallback(
    //     (value: number) => {
    //         changeMiningMode({
    //             mode: 'Custom',
    //             customCpuLevels: value,
    //             customGpuLevels: gpuValue,
    //         }).then(() => setSaved(true));
    //     },
    //     [changeMiningMode, gpuValue]
    // );

    // const handleChangeGpu = useCallback(
    //     (value: number) => {
    //         changeMiningMode({
    //             mode: 'Custom',
    //             customCpuLevels: cpuValue,
    //             customGpuLevels: value,
    //         }).then(() => setSaved(true));
    //     },
    //     [changeMiningMode, cpuValue]
    // );

    if (!maxAvailableThreads) return <LinearProgress />;

    return (
        <React.Fragment>
            <CustomLevelsHeader>
                <Typography>{t('custom-power-levels.title')}</Typography>
                <TopRightContainer>
                    <SuccessContainer $visible={isChangingMode || saved}>
                        {t('custom-power-levels.saved')}
                    </SuccessContainer>
                    <IconButton onClick={handleClose}>
                        <IoClose size={18} />
                    </IconButton>
                </TopRightContainer>
            </CustomLevelsHeader>
            <CustomLevelsContent>
                <Controller
                    control={control}
                    name={FormFields.CPU}
                    render={({ field }) => (
                        <RangeInputComponent
                            label={t('custom-power-levels.cpu-power-level')}
                            maxLevel={maxAvailableThreads.max_cpu_threads}
                            value={field.value}
                            desc={'custom-power-levels.choose-cpu-power-level'}
                            warning={t('custom-power-levels.cpu-warning')}
                            onChange={field.onChange}
                            isLoading={isChangingMode}
                        />
                    )}
                />
                <Divider />
                {fields.map((gpu, index) => (
                    <Controller
                        key={gpu.id}
                        control={control}
                        name={`${FormFields.GPUS}.${index}.gpu_name`}
                        render={({ field }) => (
                            <RangeInputComponent
                                label={`${t('custom-power-levels.gpu-power-level', { index: index + 1 })}: ${gpu.gpu_name}`}
                                maxLevel={maxAvailableThreads.max_gpus_threads[index].max_gpu_threads}
                                value={gpu.max_gpu_threads}
                                desc={'custom-power-levels.choose-gpu-power-level'}
                                warning={t('custom-power-levels.gpu-warning')}
                                onChange={(value: number) => {
                                    setValue(`${FormFields.GPUS}.${index}.max_gpu_threads`, value as never);
                                }}
                                isLoading={isChangingMode}
                            />
                        )}
                    />
                ))}
                <Divider />
                <Button onClick={handleSubmit(onSubmit)} disabled={isChangingMode}>
                    {t('custom-power-levels.save-changes')}
                </Button>
            </CustomLevelsContent>
        </React.Fragment>
    );
}
