import { describe, expect, it } from 'vitest'
import { levelFromRms } from '../audioLevel'

describe('levelFromRms', () => {
  it('draws silence and invalid input as dots', () => {
    expect(levelFromRms(0)).toBe(0)
    expect(levelFromRms(Number.NaN)).toBe(0)
    expect(levelFromRms(0.001)).toBe(0) // -60 dBFS
  })

  it('gives normal speech levels a visible height', () => {
    expect(levelFromRms(0.02)).toBeGreaterThan(0.4) // -34 dBFS
    expect(levelFromRms(0.05)).toBeGreaterThan(0.6) // -26 dBFS
  })

  it('caps loud input at full height', () => {
    expect(levelFromRms(0.2)).toBe(1)
    expect(levelFromRms(1)).toBe(1)
  })
})
