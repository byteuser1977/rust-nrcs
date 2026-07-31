/******************************************************************************
 * 交易信息表数据结构单元测试
 *
 * 验证 transaction-info-table.ts 的核心功能：
 *   - createInfoTable：数据对象 → 信息表行数组
 *     - key → label 转换（下划线→空格、首字母大写、去掉 _formatted_html 后缀）
 *     - value 类型识别（LinkRef / LinkRef[] / {html} / {quantity,decimals} / 字符串/数字/布尔）
 *     - 跳过 version.* 字段、null/undefined 值
 *     - 字母排序选项
 *   - mergeMaps：对象合并（支持排除 key）
 *
 * 对标参考：nrs.util.js NRS.createInfoTable / NRS.mergeMaps
 ******************************************************************************/
import { describe, it, expect } from 'vitest'
import { createInfoTable, mergeMaps } from '@/utils/transaction-info-table'
import type { LinkRef } from '@/utils/transaction-links'

describe('transaction-info-table', () => {
  // ==========================================================================
  // createInfoTable —— 基础转换
  // ==========================================================================
  describe('createInfoTable 基础转换', () => {
    it('空对象返回空数组', () => {
      expect(createInfoTable({})).toEqual([])
    })

    it('字符串值正确转换', () => {
      const rows = createInfoTable({ name: 'Alice' })
      expect(rows).toHaveLength(1)
      expect(rows[0]).toEqual({
        label: 'Name',
        value: 'Alice',
      })
    })

    it('数字值转换为字符串', () => {
      const rows = createInfoTable({ count: 42 })
      expect(rows[0].value).toBe('42')
    })

    it('布尔值转换为 "true"/"false" 字符串', () => {
      const rows = createInfoTable({ active: true, deleted: false })
      expect(rows.find((r) => r.label === 'Active')?.value).toBe('true')
      expect(rows.find((r) => r.label === 'Deleted')?.value).toBe('false')
    })

    it('null 值被跳过', () => {
      const rows = createInfoTable({ a: '1', b: null, c: '2' })
      expect(rows).toHaveLength(2)
      expect(rows.map((r) => r.label)).toEqual(['A', 'C'])
    })

    it('undefined 值被跳过', () => {
      const rows = createInfoTable({ a: '1', b: undefined, c: '2' })
      expect(rows).toHaveLength(2)
    })

    it('version.* 字段被跳过', () => {
      const rows = createInfoTable({
        name: 'Alice',
        'version.Message': 1,
        'version.EncryptedMessage': 1,
      })
      expect(rows).toHaveLength(1)
      expect(rows[0].label).toBe('Name')
    })
  })

  // ==========================================================================
  // createInfoTable —— key → label 转换
  // ==========================================================================
  describe('createInfoTable key → label 转换', () => {
    it('下划线替换为空格', () => {
      const rows = createInfoTable({ first_name: 'Alice' })
      expect(rows[0].label).toBe('First name')
    })

    it('首字母大写', () => {
      const rows = createInfoTable({ name: 'Alice' })
      expect(rows[0].label).toBe('Name')
    })

    it('_formatted_html 后缀被去掉', () => {
      const rows = createInfoTable({ amount_formatted_html: '<b>100</b>' })
      expect(rows[0].label).toBe('Amount')
    })

    it('多下划线字段全部替换', () => {
      const rows = createInfoTable({ total_amount_nqt: '1000' })
      expect(rows[0].label).toBe('Total amount nqt')
    })
  })

  // ==========================================================================
  // createInfoTable —— value 类型识别
  // ==========================================================================
  describe('createInfoTable value 类型识别', () => {
    it('LinkRef 对象直接保留', () => {
      const link: LinkRef = {
        type: 'account',
        value: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
        text: 'Alice',
      }
      const rows = createInfoTable({ sender: link })
      expect(rows[0].value).toEqual(link)
    })

    it('LinkRef 数组直接保留', () => {
      const links: LinkRef[] = [
        { type: 'account', value: 'NRCS-AAAA-AAAA-AAAA-AAAAA', text: 'Alice' },
        { type: 'account', value: 'NRCS-BBBB-BBBB-BBBB-BBBBB', text: 'Bob' },
      ]
      const rows = createInfoTable({ participants: links })
      expect(rows[0].value).toEqual(links)
    })

    it('_formatted_html 后缀字段识别为 HTML 值', () => {
      const rows = createInfoTable({ amount_formatted_html: '<b>100</b>' })
      expect(rows[0].value).toEqual({ html: '<b>100</b>' })
    })

    it('[quantity, decimals] 二元组转换为 { quantity, decimals }', () => {
      const rows = createInfoTable({ quantity: ['1000000', 4] })
      expect(rows[0].value).toEqual({ quantity: '1000000', decimals: 4 })
    })

    it('对象类型兜底转 JSON 字符串', () => {
      const rows = createInfoTable({ extra: { foo: 'bar' } })
      expect(rows[0].value).toBe(JSON.stringify({ foo: 'bar' }))
    })
  })

  // ==========================================================================
  // createInfoTable —— 排序选项
  // ==========================================================================
  describe('createInfoTable 排序', () => {
    it('默认保留原始顺序', () => {
      const rows = createInfoTable({ zebra: '1', apple: '2', mango: '3' })
      expect(rows.map((r) => r.label)).toEqual(['Zebra', 'Apple', 'Mango'])
    })

    it('orderAlphabetically=true 按字母排序', () => {
      const rows = createInfoTable(
        { zebra: '1', apple: '2', mango: '3' },
        true,
      )
      expect(rows.map((r) => r.label)).toEqual(['Apple', 'Mango', 'Zebra'])
    })
  })

  // ==========================================================================
  // mergeMaps
  // ==========================================================================
  describe('mergeMaps', () => {
    it('合并两个对象', () => {
      const result = mergeMaps({ a: 1, b: 2 }, { c: 3 })
      expect(result).toEqual({ a: 1, b: 2, c: 3 })
    })

    it('源对象覆盖目标对象的同名字段', () => {
      const result = mergeMaps({ a: 100 }, { a: 1, b: 2 })
      expect(result).toEqual({ a: 100, b: 2 })
    })

    it('不修改入参对象', () => {
      const source = { a: 1 }
      const target = { b: 2 }
      const result = mergeMaps(source, target)
      expect(source).toEqual({ a: 1 })
      expect(target).toEqual({ b: 2 })
      expect(result).toEqual({ a: 1, b: 2 })
    })

    it('按 excludeKeys 排除指定字段', () => {
      const result = mergeMaps(
        { a: 1, b: 2, c: 3 },
        { d: 4 },
        { a: true, c: true },
      )
      expect(result).toEqual({ b: 2, d: 4 })
    })

    it('空 excludeKeys 合并所有字段', () => {
      const result = mergeMaps({ a: 1, b: 2 }, { c: 3 }, {})
      expect(result).toEqual({ a: 1, b: 2, c: 3 })
    })

    it('空源对象返回目标对象', () => {
      const result = mergeMaps({}, { a: 1 })
      expect(result).toEqual({ a: 1 })
    })

    it('空目标对象返回源对象', () => {
      const result = mergeMaps({ a: 1 }, {})
      expect(result).toEqual({ a: 1 })
    })
  })
})
