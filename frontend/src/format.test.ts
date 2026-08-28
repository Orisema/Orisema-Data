import { describe, it, expect } from 'vitest';
import { cellText } from './format';
describe('result cell text',()=>{
  it('preserves null, booleans and exact numeric strings',()=>{
    expect(cellText(null)).toBe('NULL');
    expect(cellText(false)).toBe('false');
    expect(cellText('9007199254740993')).toBe('9007199254740993');
    expect(cellText('')).toBe('');
  });
});
