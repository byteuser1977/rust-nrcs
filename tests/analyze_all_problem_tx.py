#!/usr/bin/env python3
"""
详细分析用户提供的所有问题记录
"""

# 用户提供的原始数据（管道分隔）
data_lines = [
    "18    | -3995272460614261263 | 1440     | null                 | 0           | 100000000 | f1a52f6cddf08dc8f51f4229e536826446b15119ccc2e013fa4ff286d4b25ecc | 239    | 8070587697818876596  | 0fce4b918bfb31fc6a7ca76352225ef38c43a50c16053722b41f242fa0317908b4292196b00e8ab1f056a9d232cf64bf7d92 | 13942     | 2    | 2       | 996325769485053218  | 13953           | null                             | 0                 | FALSE  | 01f39e52171417e3df01000000000000000010a5d4e8000000                                                   | 1       | FALSE       | FALSE                 | FALSE                       | FALSE                | FALSE                   | 0               | 3488276486778630462  | FALSE                     | FALSE",
    "20    | -1481767187444870041 | 1440     | null                 | 0           | 100000000 | 67d4840098b46febad5f23b7f57813882f4e76226e9d2dd0f8f64de416f29b8a | 245    | -5790012837624046521 | efc547d77fa404efb97345aaae40426b73c461ef34e491aba4777077a26e070522603ed235f867968500b22685c70513599f | 14334     | 2    | 2       | 996325769485053218  | 14393           | null                             | 0                 | FALSE  | 01f39e52171417e3dff300000000e1f50500000000                                                           | 1       | FALSE       | FALSE                 | FALSE                       | FALSE                | FALSE                   | 0               | 3488276486778630462  | FALSE                     | FALSE",
    "23    | 522453127088588453   | 1440     | -4035311295601449920 | 0           | 100000000 | a546e8db59204007db3b0cb08312f001155a17140039b8aa51b8317d44248a82 | 276    | 8467991142738844421  | e2e54690c709d2af13302113ba7d4d57250f255d9fd40303c46e18accb474b0f39becfa2b4555046cfc41f4dbd19e818ae63 | 16075     | 1    | 0       | 996325769485053218  | 16083           | null                             | 0                 | FALSE  | 0120a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87                                   | 1       | FALSE       | FALSE                 | FALSE                       | FALSE                | FALSE                   | 0               | 3488276486778630462  | FALSE                     | TRUE",
    "43    | 7754549791140102194  | 1440     | null                 | 0           | 280000000 | 3240fc1df3b19d6b460d6c6026a732b5484b1cdb17ad45c81610b2570085e372 | 642    | -2781956316124827025 | f57800850242a989a83eed5915f30d5c12410d7cfd8dae0b11c24a445001d304fd168b1f61f2507210538a2f61bb0a273331 | 36329     | 6    | 1       | 996325769485053218  | 36588           | null                             | 0                 | FALSE  | 0115930917a7e0eabd                                                                                   | 1       | FALSE       | FALSE                 | FALSE                       | FALSE                | TRUE                    | 0               | 3488276486778630462  | FALSE                     | FALSE",
    "55    | -1077225594859266614 | 1440     | 996325769485053218   | 10000000000 | 100000000 | ca5d746307ed0cf1945e49e626a9d9ff180084b54bc267b703080af320e1ef57 | 2235   | -7558826052331724351 | 74f4f89bfe5d42d9f5368cb1167b3787190f903ce3b6935fea124c01c7f0b30458f04f0b71a4a3402fa803baeb52cd1a75b7 | 126006    | 0    | 0       | 8979268580235881594 | 126070          | null                             | 0                 | FALSE  | 0179f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93                                   | 1       | FALSE       | FALSE                 | FALSE                       | FALSE                | FALSE                   | 0               | 3488276486778630462  | FALSE                     | TRUE",
    "58    | -598422462908344986  | 15       | 996325769485053218   | 0           | 100000000 | 66d5bbc8f0f9b1f78d541cb97314e42374c8a170fb756d5d449604b3defd5189 | 2384   | 3621356438688649810  | a03910fe31d677e55320da560fd1f2a3bfe33a8099ec7cd881f818315b9b380b4ada13eb2f1c71bd65e7f7af41f8a544591d | 134030    | 2    | 10      | 996325769485053218  | 134399          | null                             | 0                 | FALSE  | 01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93 | 1       | FALSE       | FALSE                 | TRUE                        | FALSE                | FALSE                   | 1657            | -8543743489585395968 | FALSE                     | FALSE",
    "65    | 8421562545720172299  | 15       | null                 | 0           | 300000000 | 0b9f62178466df745e2b673016991327e5cc961ebd803491e22477c505669c50 | 3057   | -4050848184299982100 | 24ab6c187d676d8539cdfc1f1ae87ca050f0949b682add0680c18e275efc46069f937d40f486687c7a2282ea3e227e84e149 | 171767    | 12   | 1       | 996325769485053218  | 171769          | null                             | 0                 | FALSE  | 01e5c6099a6a80f8e0                                                                                   | 1       | FALSE       | FALSE                 | FALSE                       | FALSE                | FALSE                   | 2338            | 7386031426484071329  | FALSE                     | FALSE",
]

import struct

print("=== 问题记录详细分析 ===\n")

for line in data_lines:
    fields = [f.strip() for f in line.split('|')]
    db_id = int(fields[0])
    full_hash = fields[6]
    tx_type = int(fields[11])
    tx_subtype = int(fields[12])
    att_hex = fields[19].strip()
    has_ppm = fields[26].strip()  # HAS_PRUNABLE_MESSAGE
    
    # 确保十六进制字符串长度为偶数
    if len(att_hex) % 2 != 0:
        att_hex = att_hex[:-1]  # 移除最后一个字符
    
    att_bytes = bytes.fromhex(att_hex)
    
    print(f"DB_ID={db_id}, Type={tx_type}:{tx_subtype}")
    print(f"  FULL_HASH: {full_hash}")
    print(f"  ATTACHMENT_BYTES ({len(att_bytes)}B): {att_hex}")
    print(f"  HAS_PRUNABLE_MESSAGE: {has_ppm}")
    
    # 根据类型解析
    if tx_type == 2 and tx_subtype == 2:
        # OrderPlacement
        if len(att_bytes) == 25:
            version = att_bytes[0]
            asset = struct.unpack_from('<q', att_bytes, 1)[0]
            qty = struct.unpack_from('<q', att_bytes, 9)[0]
            price = struct.unpack_from('<q', att_bytes, 17)[0]
            print(f"  解析: version={version}, asset={asset}, qty={qty}, price={price}")
        else:
            print(f"  异常: 长度 {len(att_bytes)} 不是标准 25 字节")
    
    elif tx_type == 1 and tx_subtype == 0:
        # ArbitraryMessage with PrunablePlainMessage
        print(f"  解析: PrunablePlainMessage hash (version=1 + 32B hash)")
    
    elif tx_type == 6 and tx_subtype == 1:
        # TaggedDataExtend
        if len(att_bytes) == 9:
            version = att_bytes[0]
            tagged_id = struct.unpack_from('<q', att_bytes, 1)[0]
            print(f"  解析: version={version}, taggedDataId={tagged_id}")
    
    elif tx_type == 0 and tx_subtype == 0:
        # Payment with PrunablePlainMessage
        print(f"  解析: Payment with PrunablePlainMessage hash")
    
    elif tx_type == 2 and tx_subtype == 10:
        # AssetPropertySet
        print(f"  解析: AssetPropertySet + PublicKeyAnnouncement")
    
    elif tx_type == 12 and tx_subtype == 1:
        # ContractReferenceDelete
        if len(att_bytes) == 9:
            version = att_bytes[0]
            ref_id = struct.unpack_from('<q', att_bytes, 1)[0]
            print(f"  解析: version={version}, contractReferenceId={ref_id}")
    
    print()
