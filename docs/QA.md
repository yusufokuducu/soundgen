# SoundGen - QA report

Generated: 2026-10-24 07:25

## Duration / peak / LUFS / loop seam

```
id                                dur    peak    lufs frames    seam
weapon_pistol_fire               0.45   -1.00   -20.6  21600       -
weapon_rifle_fire                0.77   -1.00   -24.3  36960       -
weapon_shotgun_fire              1.00   -1.00   -25.0  48000       -
weapon_smg_fire                  0.33   -1.00   -22.8  15841       -
weapon_suppressed_fire           0.47   -1.00   -23.1  22560       -
weapon_dry_fire                  0.21   -1.00   -24.4  10080       -
weapon_reload_mag                0.67   -1.00   -21.0  32160       -
weapon_casing_drop               0.75   -1.00   -20.9  36000       -
weapon_bolt_pull                 0.65   -1.00   -21.6  31201       -
weapon_shell_insert              0.60   -1.00   -22.6  28801       -
zombie_growl_low                 1.55   -1.00   -17.5  74400       -
zombie_growl_high                1.05   -1.00   -17.9  50400       -
zombie_scream                    1.75   -1.00   -15.3  84000       -
zombie_pain_hit                  0.50   -1.00   -17.5  24000       -
zombie_attack_lunge              1.15   -1.00   -18.2  55200       -
zombie_bite                      0.55   -1.00   -21.6  26400       -
zombie_death_rattle              2.45   -2.00   -16.8 117600       -
zombie_idle_breath               2.75   -5.00   -21.0 132000       -
zombie_horde_bed                 3.65   -4.00   -20.1 175200       -
gore_flesh_hit                   0.39   -1.00   -20.6  18720       -
gore_flesh_slash                 0.33   -1.00   -22.0  15841       -
gore_bone_break                  0.35   -1.00   -23.2  16801       -
gore_head_pop                    0.51   -1.00   -19.5  24480       -
gore_body_fall                   0.65   -1.00   -19.0  31201       -
gore_blood_splat                 0.60   -1.00   -23.0  28801       -
gore_dismember                   0.57   -1.00   -20.3  27360       -
gore_zombie_crunch               0.49   -1.00   -18.9  23520       -
foley_footstep_concrete          0.31   -1.00   -21.7  14880       -
foley_footstep_grass             0.29   -1.00   -22.5  13920       -
foley_footstep_mud               0.31   -1.00   -20.6  14880       -
foley_footstep_metal             0.37   -1.00   -21.9  17760       -
foley_body_land                  0.83   -1.00   -21.2  39840       -
foley_cloth_rustle               0.47   -1.00   -21.7  22560       -
foley_door_creak                 2.15   -1.00   -17.0 103200       -
foley_debris_glass               0.67   -1.00   -22.7  32160       -
loop_propeller_small             4.00   -6.00   -16.3 192000    0.15
loop_propeller_large             6.00   -6.00   -17.8 288000    0.01
loop_drone_motor                 4.00   -6.00   -19.6 192000    0.57
loop_helicopter_far              6.67   -9.00   -23.9 320000    0.09
loop_wind_loop                   8.00   -8.00   -28.8 384000    0.00
loop_campfire                    8.00   -8.00   -30.1 384000    0.00
crate_wood_creak                 1.40   -1.00   -16.8  67200       -
crate_open_pop                   0.60   -1.00   -15.5  28801       -
crate_shatter                    1.15   -1.00   -20.7  55200       -
crate_lock_click                 0.20   -1.00   -16.3   9600       -
crate_reward_reveal              1.55   -1.00   -12.7  74400       -
crate_metal_open                 0.95   -1.00   -12.7  45600       -
celebrate_confetti_popper        1.35   -1.00   -26.3  64800       -
celebrate_party_horn             0.85   -1.00   -21.2  40800       -
celebrate_confetti_rain          3.05   -1.00   -27.2 146400       -
celebrate_fanfare_sting          1.05   -1.00   -13.7  50400       -
celebrate_coin_burst             0.95   -1.00   -23.0  45600       -
celebrate_level_up               1.75   -1.00   -16.6  84000       -
ui_click_soft                    0.18   -5.00   -27.7   8640       -
ui_click_hard                    0.25   -3.50   -24.4  12000       -
ui_hover_tick                    0.12   -6.00   -31.4   5760       -
ui_coin_pickup                   0.55   -4.00   -19.9  26400       -
ui_purchase                      0.60   -4.00   -24.3  28801       -
ui_error_buzz                    0.40   -4.50   -22.2  19200       -
ui_level_up_short                0.55   -4.00   -19.1  26400       -
ui_menu_whoosh                   0.33   -5.00   -22.9  15841       -
amb_wind_ruined_city             6.05   -3.00   -19.9 290400       -
amb_distant_battle               6.55   -3.00   -19.2 314400       -
amb_zombie_moans_far             7.05   -3.00   -17.9 338400       -
amb_underground_hum              6.05   -3.00   -18.8 290400       -
amb_fire_crackle                 5.05   -3.00   -26.1 242401       -

66 presets, total render 1213 ms, avg 18.4 ms
```

## Envelope and band split

```
id                             atk_ms dec_ms  crest   low%   mid%    hi%  air%
weapon_pistol_fire                3.2  203.0   19.4    73    23     3     1
weapon_rifle_fire                 2.8  283.2   22.8    53    40     5     2
weapon_shotgun_fire               3.1  488.2   21.9    76    23     1     0
weapon_smg_fire                   3.1   71.5   21.1    60    30     7     3
weapon_suppressed_fire            4.5  142.8   22.1    63    31     5     0
weapon_dry_fire                   0.7   26.8   22.7    10    54    28     8
weapon_reload_mag                 0.2  467.6   20.5    12    47    37     4
weapon_casing_drop                0.2  395.3   19.3     1    44    49     6
weapon_bolt_pull                120.3  301.3   19.7     3    58    35     4
weapon_shell_insert               0.6  312.0   21.0    39    45    15     1
zombie_growl_low                 43.1 1025.3   15.9    68    30     2     0
zombie_growl_high                32.0  740.2   16.2    55    39     5     1
zombie_scream                   149.0 1222.3   14.0    13    72    14     2
zombie_pain_hit                  17.1  309.6   16.7    55    41     3     0
zombie_attack_lunge              66.9  771.0   17.0    55    40     4     0
zombie_bite                       3.3  203.5   21.3    24    62    12     1
zombie_death_rattle             505.2 1482.3   14.3    76    23     1     0
zombie_idle_breath              958.8 1424.1   15.7    42    49     8     1
zombie_horde_bed                794.0 2523.5   15.7    73    26     2     0
gore_flesh_hit                    3.0  116.1   18.9    66    30     3     0
gore_flesh_slash                 14.6  119.6   20.3    38    35    21     7
gore_bone_break                   2.8  124.8   21.5     1    35    53    11
gore_head_pop                     3.7  252.8   18.9    83    15     2     0
gore_body_fall                    5.4  399.5   16.8    88    12     1     0
gore_blood_splat                  7.2  370.4   20.4    53    41     6     0
gore_dismember                   12.8  303.0   20.1     4    59    32     4
gore_zombie_crunch                1.2  262.2   18.1    11    66    21     1
foley_footstep_concrete           1.4   28.0   20.0    48    47     4     1
foley_footstep_grass             24.4   85.3   20.8    16    47    32     5
foley_footstep_mud                6.9   84.4   18.9    57    41     2     0
foley_footstep_metal              3.6   90.0   20.2    12    75    12     1
foley_body_land                   6.5  269.0   18.1    89    10     1     0
foley_cloth_rustle               28.9  222.7   20.8     2    45    45     7
foley_door_creak                 16.1 1535.0   15.6    65    32     2     0
foley_debris_glass               86.1  443.5   21.2     2    50    38    10
loop_propeller_small           3299.8  700.2    9.7    84    14     2     0
loop_propeller_large           1847.1 4152.9   11.1    91     8     1     0
loop_drone_motor               2794.8 1205.2   13.0    30    56    12     3
loop_helicopter_far               4.0 6662.6   14.3    77    21     1     0
loop_wind_loop                    4.0 7994.0   20.1    76    22     2     0
loop_campfire                     4.0 7993.0   21.4    82    18     0     0
crate_wood_creak                  0.8  883.3   14.1    41    57     2     0
crate_open_pop                    0.2  431.2   13.0    52    46     2     0
crate_shatter                     0.3  472.6   18.0    19    75     6     0
crate_lock_click                  1.0  141.3   14.6    20    73     6     1
crate_reward_reveal             500.5 1049.5   12.0     5    77    17     1
crate_metal_open                  1.0  760.6   11.6    19    73     7     0
celebrate_confetti_popper         1.2  587.4   23.8     2    30    53    15
celebrate_party_horn             12.3  488.8   19.1    41    54     5     1
celebrate_confetti_rain         699.9 1125.0   25.7     2    39    52     8
celebrate_fanfare_sting         265.8  535.2   12.4    11    76    12     1
celebrate_coin_burst              0.5  402.2   21.7     1    53    41     5
celebrate_level_up              627.7  826.1   15.8    11    67    19     3
ui_click_soft                     1.2   17.3   22.0     7    67    25     2
ui_click_hard                     0.5   33.1   20.2    20    62    17     2
ui_hover_tick                     1.3   12.6   24.8     2    47    45     6
ui_coin_pickup                    1.0  251.0   16.6     3    69    24     3
ui_purchase                      91.9  159.4   18.5     5    70    23     2
ui_error_buzz                    16.8  217.5   17.0    61    36     2     0
ui_level_up_short               123.5  186.9   15.8     4    71    23     1
ui_menu_whoosh                   34.3  186.7   17.2    26    59    13     2
amb_wind_ruined_city            656.2 4132.8   16.4    76    22     2     0
amb_distant_battle              942.2 4612.1   15.7    91     9     0     0
amb_zombie_moans_far           1292.7 5254.6   14.4    75    23     2     0
amb_underground_hum            2650.7 2938.2   15.3    67    28     5     0
amb_fire_crackle                851.6 2212.1   22.8    79    19     2     0
```

## Automatic checks

- [x] no NaN / infinite samples
- [x] peak at or below 0 dBFS (limiter + normalisation)
- [x] loop seam below 0.05
- [x] unique ids (no duplicates)

