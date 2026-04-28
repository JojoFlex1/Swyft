import { Injectable } from '@nestjs/common';
import { PoolListQuery, PoolListResult, PoolSnapshot, TickData, GetTicksQuery } from './pool.types';
import { PrismaService } from '../prisma/prisma.service';

type PoolStatePatch = {
  currentPrice?: string;
};

@Injectable()
export class PoolsRepository {
  private readonly pools = new Map<string, PoolSnapshot>();

  constructor(private readonly prisma: PrismaService) {}

  async poolExists(id: string): Promise<boolean> {
    const count = await this.prisma.poolCreated.count({ where: { poolId: id } });
    return count > 0;
  }

  async listActivePools(query: PoolListQuery): Promise<PoolListResult> {
    const search = query.search?.trim().toLowerCase();

    const filtered = [...this.pools.values()]
      .filter((pool) => pool.active)
      .filter((pool) => {
        if (!search) return true;
        return (
          pool.token0.toLowerCase().includes(search) ||
          pool.token1.toLowerCase().includes(search)
        );
      });

    const sorted = filtered.sort((a, b) => {
      if (query.orderBy === 'volume') return b.volume24h - a.volume24h;
      if (query.orderBy === 'apr') return b.feeApr - a.feeApr;
      return b.tvl - a.tvl;
    });

    const offset = (query.page - 1) * query.limit;
    const items = sorted.slice(offset, offset + query.limit);

    return {
      items,
      total: sorted.length,
    };
  }

  async upsertPoolState(poolId: string, patch: PoolStatePatch): Promise<void> {
    const existing = this.pools.get(poolId);
    if (!existing) return;

    const currentPrice = patch.currentPrice
      ? Number.parseFloat(patch.currentPrice)
      : existing.currentPrice;

    this.pools.set(poolId, {
      ...existing,
      currentPrice,
      updatedAt: Date.now(),
    });
  }

  async getTicks(query: GetTicksQuery): Promise<TickData[]> {
    const where: any = {
      poolId: query.poolId,
    };

    if (query.lowerTick !== undefined && query.upperTick !== undefined) {
      where.tickIndex = {
        gte: query.lowerTick,
        lte: query.upperTick,
      };
    } else if (query.lowerTick !== undefined) {
      where.tickIndex = {
        gte: query.lowerTick,
      };
    } else if (query.upperTick !== undefined) {
      where.tickIndex = {
        lte: query.upperTick,
      };
    }

    const ticks = await this.prisma.tick.findMany({
      where,
      orderBy: {
        tickIndex: 'asc',
      },
      select: {
        tickIndex: true,
        liquidityNet: true,
        liquidityGross: true,
        feeGrowthOutside0X128: true,
        feeGrowthOutside1X128: true,
      },
    });

    return ticks.map((tick) => ({
      tickIndex: tick.tickIndex,
      liquidityNet: tick.liquidityNet,
      liquidityGross: tick.liquidityGross,
      feeGrowthOutside0X128: tick.feeGrowthOutside0X128,
      feeGrowthOutside1X128: tick.feeGrowthOutside1X128,
    }));
  }
}
