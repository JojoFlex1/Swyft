import { Type } from 'class-transformer';
import {
  IsInt,
  IsOptional,
} from 'class-validator';

export class GetTicksQueryDto {
  @Type(() => Number)
  @IsInt()
  @IsOptional()
  lowerTick?: number;

  @Type(() => Number)
  @IsInt()
  @IsOptional()
  upperTick?: number;
}
