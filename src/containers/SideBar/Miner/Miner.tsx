import Tile from './components/Tile.tsx';
import { MinerContainer, TileContainer, Unit } from './styles.ts';

import ModeSelect from './components/ModeSelect.tsx';
import { formatNumber } from '@app/utils/formatNumber.ts';

import { useMiningStore } from '@app/store/useMiningStore.ts';
import { ExpandableTile } from '@app/containers/SideBar/Miner/components/ExpandableTile.tsx';
import formatBalance from '@app/utils/formatBalance.ts';
import { Typography } from '@app/components/elements/Typography.tsx';
import {
    ExpandableTileItem,
    ExpandedContentTile,
} from '@app/containers/SideBar/Miner/components/ExpandableTile.styles.ts';
import { useShallow } from 'zustand/react/shallow';

import { useAppConfigStore } from '@app/store/useAppConfigStore.ts';
import { useHardwareStats } from '@app/hooks/useHardwareStats.ts';
import useMiningStatesSync from '@app/hooks/mining/useMiningStatesSync.ts';
import { Trans, useTranslation } from 'react-i18next';

export default function Miner() {
    const { t } = useTranslation('mining-view', { useSuspense: false });
    useMiningStatesSync();
    const { cpu: cpuHardwareStats, gpu: gpuHardwareStats } = useHardwareStats();
    const miningInitiated = useMiningStore((s) => s.miningInitiated);
    const isCpuMiningEnabled = useAppConfigStore((s) => s.cpu_mining_enabled);
    const isGpuMiningEnabled = useAppConfigStore((s) => s.gpu_mining_enabled);
    const { cpu_estimated_earnings, cpu_hash_rate, cpu_is_mining } = useMiningStore(
        useShallow((s) => ({
            cpu_estimated_earnings: s.cpu.mining.estimated_earnings,
            cpu_hash_rate: s.cpu.mining.hash_rate,
            cpu_is_mining: s.cpu.mining.is_mining,
        }))
    );
    const { gpu_estimated_earnings, gpu_hash_rate, gpu_is_mining } = useMiningStore(
        useShallow((s) => ({
            gpu_estimated_earnings: s.gpu.mining.estimated_earnings,
            gpu_hash_rate: s.gpu.mining.hash_rate,
            gpu_is_mining: s.gpu.mining.is_mining,
        }))
    );

    const isMiningInProgress = cpu_is_mining || gpu_is_mining;

    const isWaitingForCPUHashRate = isCpuMiningEnabled && cpu_is_mining && cpu_hash_rate <= 0;
    const isWaitingForGPUHashRate = isGpuMiningEnabled && gpu_is_mining && gpu_hash_rate <= 0;
    const isLoading = (miningInitiated && !isMiningInProgress) || (isMiningInProgress && !miningInitiated);

    const totalEarnings = cpu_estimated_earnings + gpu_estimated_earnings;
    const earningsLoading = totalEarnings <= 0 && (isWaitingForCPUHashRate || isWaitingForGPUHashRate);

    const gpuChipValue = gpuHardwareStats
        ? gpuHardwareStats?.reduce((acc, current) => acc + current.usage_percentage, 0) /
          (gpuHardwareStats?.length || 1)
        : 0;

    return (
        <MinerContainer>
            <TileContainer>
                <Tile
                    title="CPU Power"
                    stats={isCpuMiningEnabled && cpu_is_mining ? formatNumber(cpu_hash_rate) : '-'}
                    isLoading={isCpuMiningEnabled && (isLoading || isWaitingForCPUHashRate)}
                    chipValue={cpuHardwareStats?.usage_percentage}
                    unit="H/s"
                    useLowerCase
                />
                <Tile
                    title="GPU Power"
                    stats={isGpuMiningEnabled && gpu_is_mining ? formatNumber(gpu_hash_rate) : '-'}
                    isLoading={isGpuMiningEnabled && (isLoading || isWaitingForGPUHashRate)}
                    chipValue={gpuChipValue}
                    unit="H/s"
                    useLowerCase
                />
                <ModeSelect />
                <ExpandableTile
                    title="Est tXTM/day"
                    stats={isMiningInProgress && Number.isFinite(totalEarnings) ? formatBalance(totalEarnings) : '-'}
                    isLoading={earningsLoading}
                    useLowerCase
                >
                    <Typography variant="h5" style={{ color: '#000' }}>
                        {t('estimated-earnings')}
                    </Typography>
                    <Typography>{t('you-earn-rewards-separately')}</Typography>
                    <ExpandedContentTile>
                        <Typography>CPU {t('estimated-earnings')}</Typography>
                        <ExpandableTileItem>
                            <Typography
                                variant="h5"
                                style={{
                                    textTransform: 'lowercase',
                                    fontWeight: 500,
                                    lineHeight: '1.02',
                                }}
                            >
                                {isMiningInProgress && isCpuMiningEnabled ? formatBalance(cpu_estimated_earnings) : '-'}
                            </Typography>
                            <Unit>
                                <Typography>
                                    <Trans>tXTM/</Trans>
                                    {t('day')}
                                </Typography>
                            </Unit>
                        </ExpandableTileItem>
                    </ExpandedContentTile>
                    <ExpandedContentTile>
                        <Typography>GPU {t('estimated-earnings')}</Typography>
                        <ExpandableTileItem>
                            <Typography
                                variant="h5"
                                style={{
                                    textTransform: 'lowercase',
                                    fontWeight: 500,
                                    lineHeight: '1.02',
                                }}
                            >
                                {isMiningInProgress && isGpuMiningEnabled ? formatBalance(gpu_estimated_earnings) : '-'}
                            </Typography>
                            <Unit>
                                <Typography>
                                    <Trans>tXTM/</Trans>
                                    {t('day')}
                                </Typography>
                            </Unit>
                        </ExpandableTileItem>
                    </ExpandedContentTile>
                </ExpandableTile>
            </TileContainer>
        </MinerContainer>
    );
}
