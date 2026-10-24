# SoundGen - QA report

Generated: 2026-10-24 08:10

## Duration / peak / LUFS / loop seam

```
id                                dur    peak    lufs frames    seam
weapon_pistol_fire               0.45   -1.00   -20.6  21600       -
weapon_rifle_fire                0.77   -1.00   -24.4  36960       -
weapon_shotgun_fire              1.00   -1.00   -25.0  48000       -
weapon_smg_fire                  0.33   -1.00   -22.2  15841       -
weapon_suppressed_fire           0.47   -1.00   -22.2  22560       -
weapon_dry_fire                  0.21   -1.00   -28.1  10080       -
weapon_reload_mag                0.67   -1.00   -23.0  32160       -
weapon_casing_drop               0.75   -1.00   -25.3  36000       -
weapon_bolt_pull                 0.65   -1.00   -24.7  31201       -
weapon_shell_insert              0.60   -1.00   -22.3  28801       -
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
gore_bone_break                  0.35   -1.00   -25.5  16801       -
gore_head_pop                    0.51   -1.00   -19.6  24480       -
gore_body_fall                   0.65   -1.00   -19.5  31201       -
gore_blood_splat                 0.60   -1.00   -23.0  28801       -
gore_dismember                   0.57   -1.00   -20.3  27360       -
gore_zombie_crunch               0.49   -1.00   -18.9  23520       -
foley_footstep_concrete          0.31   -1.00   -27.3  14880       -
foley_footstep_grass             0.29   -1.00   -22.5  13920       -
foley_footstep_mud               0.31   -1.00   -20.6  14880       -
foley_footstep_metal             0.37   -1.00   -25.1  17760       -
foley_body_land                  0.83   -1.00   -21.3  39840       -
foley_cloth_rustle               0.47   -1.00   -21.8  22560       -
foley_door_creak                 2.15   -1.00   -17.0 103200       -
foley_debris_glass               0.67   -1.00   -27.3  32160       -
loop_propeller_small             4.00   -6.00   -16.3 192000    0.15
loop_propeller_large             6.00   -6.00   -17.8 288000    0.01
loop_drone_motor                 4.00   -6.00   -19.6 192000    0.57
loop_helicopter_far              6.67   -9.00   -23.9 320000    0.09
loop_wind_loop                   8.00   -8.00   -28.8 384000    0.00
loop_campfire                    8.00   -8.00   -30.1 384000    0.00
crate_wood_creak                 1.40   -1.00   -21.9  67200       -
crate_open_pop                   0.60   -1.00   -22.3  28801       -
crate_shatter                    1.15   -1.00   -25.9  55200       -
crate_lock_click                 0.20   -1.00   -25.2   9600       -
crate_reward_reveal              1.55   -1.00   -16.6  74400       -
crate_metal_open                 0.95   -1.00   -19.9  45600       -
celebrate_confetti_popper        1.35   -1.00   -26.8  64800       -
celebrate_party_horn             0.85   -1.00   -21.2  40800       -
celebrate_confetti_rain          3.05   -1.00   -27.6 146400       -
celebrate_fanfare_sting          1.05   -1.00   -13.6  50400       -
celebrate_coin_burst             0.95   -1.00   -23.0  45600       -
celebrate_level_up               1.75   -1.00   -16.6  84000       -
ui_click_soft                    0.18   -5.00   -27.4   8640       -
ui_click_hard                    0.25   -3.50   -25.2  12000       -
ui_hover_tick                    0.12   -6.00   -32.1   5760       -
ui_coin_pickup                   0.55   -4.00   -19.9  26400       -
ui_purchase                      0.60   -4.00   -22.2  28801       -
ui_error_buzz                    0.40   -4.50   -22.2  19200       -
ui_level_up_short                0.55   -4.00   -19.1  26400       -
ui_menu_whoosh                   0.33   -5.00   -22.9  15841       -
amb_wind_ruined_city             6.05   -3.00   -19.9 290400       -
amb_distant_battle               6.55   -3.00   -19.2 314400       -
amb_zombie_moans_far             7.05   -3.00   -17.9 338400       -
amb_underground_hum              6.05   -3.00   -19.0 290400       -
amb_fire_crackle                 5.05   -3.00   -21.3 242401       -

66 presets, total render 1746 ms, avg 26.4 ms
```

## Envelope and band split

```
id                             atk_ms dec_ms  crest   low%   mid%    hi%  air%
weapon_pistol_fire                3.2  203.0   19.4    74    22     2     1
weapon_rifle_fire                 2.8  283.2   22.9    54    40     4     2
weapon_shotgun_fire               3.1  488.2   21.9    76    23     1     0
weapon_smg_fire                   3.1   71.3   20.5    63    30     4     3
weapon_suppressed_fire            4.5  197.7   21.2    67    29     4     0
weapon_dry_fire                   0.7   15.8   26.4     5    26    47    23
weapon_reload_mag                 0.4  406.7   22.8    45    35    17     3
weapon_casing_drop                0.2  361.2   23.6     1    41    49     9
weapon_bolt_pull                120.3  255.4   23.1     3    50    41     6
weapon_shell_insert               5.3  305.1   20.6    56    40     3     0
zombie_growl_low                 43.1 1025.3   15.9    68    30     2     0
zombie_growl_high                32.0  740.2   16.2    55    39     5     1
zombie_scream                   149.0 1222.3   14.0    13    72    14     2
zombie_pain_hit                  17.1  309.6   16.7    55    41     3     0
zombie_attack_lunge              66.9  771.0   17.0    55    40     4     0
zombie_bite                       3.3  203.5   21.2    24    62    12     1
zombie_death_rattle             505.2 1482.3   14.3    76    23     1     0
zombie_idle_breath              958.8 1424.1   15.7    42    49     8     1
zombie_horde_bed                794.0 2523.5   15.7    73    26     2     0
gore_flesh_hit                    3.0  116.1   18.9    66    30     3     0
gore_flesh_slash                 14.6  119.6   20.3    38    35    21     7
gore_bone_break                   0.9  117.9   23.8     2    23    56    19
gore_head_pop                     3.7  252.8   18.9    84    14     2     0
gore_body_fall                    5.4  301.9   17.1    91     9     0     0
gore_blood_splat                  7.2  370.4   20.4    53    41     6     0
gore_dismember                   12.8  302.9   20.2     4    59    32     4
gore_zombie_crunch               85.9  216.6   18.1    56    36     7     1
foley_footstep_concrete           1.2   35.9   25.6     2    22    48    28
foley_footstep_grass             24.4   85.3   20.8    16    47    32     5
foley_footstep_mud                6.9   84.4   18.9    57    41     2     0
foley_footstep_metal              0.7   60.6   23.4    61    30     7     2
foley_body_land                   6.5  269.1   18.1    89    10     1     0
foley_cloth_rustle               28.9  222.7   20.8     2    45    45     7
foley_door_creak                 16.1 1535.0   15.6    65    32     2     0
foley_debris_glass                1.2  523.6   25.5     1    22    51    25
loop_propeller_small           3299.8  700.2    9.7    84    14     2     0
loop_propeller_large           1847.1 4152.9   11.1    91     8     1     0
loop_drone_motor               2794.8 1205.2   13.0    30    56    12     3
loop_helicopter_far               4.0 6662.6   14.3    77    21     1     0
loop_wind_loop                    4.0 7994.0   20.1    76    22     2     0
loop_campfire                     4.0 7993.0   21.4    82    18     0     0
crate_wood_creak                  4.1  849.3   19.2    31    64     4     0
crate_open_pop                    2.1  276.4   19.5    31    62     6     1
crate_shatter                     1.3  359.1   23.2     7    69    22     2
crate_lock_click                  1.3   64.6   23.5     6    46    41     8
crate_reward_reveal             511.4  727.0   15.7     5    69    24     2
crate_metal_open                  2.9  520.8   18.1    17    76     6     0
celebrate_confetti_popper         1.2  582.3   24.1     2    30    53    16
celebrate_party_horn             12.3  488.8   19.1    41    54     5     1
celebrate_confetti_rain         699.9 1125.0   26.0     2    39    51     8
celebrate_fanfare_sting         272.2  534.5   12.3    11    76    11     1
celebrate_coin_burst             91.1  312.3   21.8     1    40    49    10
celebrate_level_up              627.7  826.1   15.8    11    67    19     3
ui_click_soft                     1.2   19.8   21.7    14    71    13     1
ui_click_hard                     1.6   33.7   21.0    37    54     8     2
ui_hover_tick                     1.3   12.6   25.4     2    49    42     7
ui_coin_pickup                    1.0  250.4   16.6     3    70    24     3
ui_purchase                      91.9  172.3   16.4     5    71    22     1
ui_error_buzz                    16.8  217.5   17.0    61    36     2     0
ui_level_up_short               123.5  186.9   15.8     4    71    23     1
ui_menu_whoosh                   34.3  186.7   17.2    26    59    13     2
amb_wind_ruined_city            656.2 4132.8   16.4    76    22     2     0
amb_distant_battle              942.2 4612.1   15.7    91     9     0     0
amb_zombie_moans_far           1292.7 5254.6   14.4    75    23     2     0
amb_underground_hum            2870.7 2608.2   15.4    81    17     1     0
amb_fire_crackle                657.0 2928.5   18.0    83    16     1     0
```

## Automatic checks

- [x] no NaN / infinite samples
- [x] peak at or below 0 dBFS (limiter + normalisation)
- [x] loop seam below 0.05
- [x] unique ids (no duplicates)

