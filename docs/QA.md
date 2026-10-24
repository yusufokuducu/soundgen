# SoundGen - QA report

Generated: 2026-10-24 07:10

## Duration / peak / LUFS / loop seam

```
id                                dur    peak    lufs frames    seam
weapon_pistol_fire               0.45   -1.00   -21.2  21600       -
weapon_rifle_fire                0.77   -1.00   -24.8  36960       -
weapon_shotgun_fire              1.00   -1.00   -25.1  48000       -
weapon_smg_fire                  0.33   -1.00   -24.0  15841       -
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
zombie_horde_bed                 3.65   -4.00   -20.2 175200       -
gore_flesh_hit                   0.39   -1.00   -20.7  18720       -
gore_flesh_slash                 0.33   -1.00   -21.9  15841       -
gore_bone_break                  0.35   -1.00   -24.5  16801       -
gore_head_pop                    0.51   -1.00   -17.5  24480       -
gore_body_fall                   0.65   -1.00   -19.0  31201       -
gore_blood_splat                 0.60   -1.00   -22.7  28801       -
gore_dismember                   0.57   -1.00   -20.3  27360       -
gore_zombie_crunch               0.49   -1.00   -19.0  23520       -
foley_footstep_concrete          0.31   -1.00   -21.7  14880       -
foley_footstep_grass             0.29   -1.00   -22.5  13920       -
foley_footstep_mud               0.31   -1.00   -20.0  14880       -
foley_footstep_metal             0.37   -1.00   -21.6  17760       -
foley_body_land                  0.83   -1.00   -21.1  39840       -
foley_cloth_rustle               0.47   -1.00   -21.7  22560       -
foley_door_creak                 2.15   -1.00   -17.0 103200       -
foley_debris_glass               0.67   -1.00   -22.7  32160       -
loop_propeller_small             4.05   -6.00   -16.4 194401   0.000
loop_propeller_large             6.05   -6.00   -19.4 290400   0.000
loop_drone_motor                 4.05   -6.00   -22.5 194401   0.034
loop_helicopter_far              6.72   -9.00   -22.4 322400   0.000
loop_wind_loop                   8.05   -8.00   -21.8 386400   0.000
loop_campfire                    8.05   -8.00   -21.4 386400   0.000
crate_wood_creak                 1.55   -1.00   -16.8  74400       -
crate_open_pop                   0.85   -1.00   -17.3  40800       -
crate_shatter                    1.35   -1.00   -21.8  64800       -
crate_lock_click                 0.27   -1.00   -17.7  12961       -
crate_reward_reveal              1.75   -1.00   -13.3  84000       -
crate_metal_open                 1.15   -1.00   -13.9  55200       -
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
amb_wind_ruined_city             6.05   -3.00   -19.6 290400       -
amb_distant_battle               6.55   -3.00   -19.5 314400       -
amb_zombie_moans_far             7.05   -3.00   -18.1 338400       -
amb_underground_hum              6.05   -3.00   -18.8 290400       -
amb_fire_crackle                 5.05   -3.00   -26.1 242401       -

66 presets, total render 1162 ms, avg 17.6 ms
```

## Envelope and band split

```
id                             atk_ms dec_ms  crest   low%   mid%    hi%  air%
weapon_pistol_fire                3.2  202.2   20.0    72    22     4     2
weapon_rifle_fire                 2.8  298.1   23.3    49    41     7     3
weapon_shotgun_fire               3.3  431.3   21.9    77    22     2     0
weapon_smg_fire                   3.1   58.4   22.3    53    30    12     6
weapon_suppressed_fire            4.5  142.8   22.1    63    31     5     0
weapon_dry_fire                   0.7   26.8   22.7    10    54    28     8
weapon_reload_mag                 0.2  467.6   20.5    12    47    37     4
weapon_casing_drop                0.2  395.3   19.3     1    44    49     6
weapon_bolt_pull                120.3  301.3   19.7     3    58    35     4
weapon_shell_insert               0.6  312.0   21.0    39    45    15     1
zombie_growl_low                 43.1 1025.3   15.9    68    30     2     0
zombie_growl_high                32.0  740.2   16.2    55    39     5     1
zombie_scream                   149.0 1222.0   14.0    13    72    14     2
zombie_pain_hit                  17.1  309.6   16.8    55    41     3     0
zombie_attack_lunge              66.9  771.0   17.0    55    40     4     0
zombie_bite                       3.3  203.4   21.3    24    62    12     1
zombie_death_rattle             505.2 1482.0   14.3    76    23     1     0
zombie_idle_breath              909.5 1473.4   15.8    42    49     9     1
zombie_horde_bed                794.0 2404.5   15.7    73    26     2     0
gore_flesh_hit                   10.4  147.6   19.0    64    32     4     0
gore_flesh_slash                  3.3  105.4   20.2    39    33    21     7
gore_bone_break                   0.9  123.0   22.9     2    32    52    14
gore_head_pop                     0.7  306.6   16.9    86    12     1     0
gore_body_fall                    5.4  399.5   16.8    88    12     1     0
gore_blood_splat                  7.2  335.5   20.0    64    33     3     0
gore_dismember                   12.8  303.0   20.1     4    59    32     4
gore_zombie_crunch                1.2  262.3   18.1    11    66    21     1
foley_footstep_concrete           1.4   28.1   20.0    48    47     4     1
foley_footstep_grass             24.4   85.3   20.8    16    47    32     5
foley_footstep_mud                6.9   84.1   18.3    59    39     1     0
foley_footstep_metal              3.6   97.3   19.9     9    78    13     1
foley_body_land                   6.5  250.9   18.0    90     9     0     0
foley_cloth_rustle               28.9  222.8   20.8     2    45    45     7
foley_door_creak                 16.1 1535.1   15.5    65    32     2     0
foley_debris_glass               86.1  443.5   21.2     2    50    38    10
loop_propeller_small           3299.8  692.7    9.8    83    15     2     0
loop_propeller_large             11.2 5971.9   12.8    90     9     1     0
loop_drone_motor                 10.0 3978.1   15.9    29    57    12     3
loop_helicopter_far             655.9 5991.8   12.8    72    25     2     0
loop_wind_loop                 5254.5 2727.7   13.1    76    22     2     0
loop_campfire                  5255.0 2725.9   12.8    82    18     0     0
crate_wood_creak                  1.1  882.9   14.6    41    56     2     0
crate_open_pop                    1.0  430.5   14.6    53    45     2     0
crate_shatter                     1.0  472.0   18.9    20    75     5     0
crate_lock_click                  1.0  142.7   16.0    21    73     5     1
crate_reward_reveal             500.5 1049.7   12.4     5    77    17     1
crate_metal_open                  1.0  760.6   12.5    19    73     7     0
celebrate_confetti_popper         1.2  587.4   23.8     2    30    53    15
celebrate_party_horn             12.3  488.9   19.1    41    54     5     1
celebrate_confetti_rain         699.9 1125.0   25.7     1    39    52     8
celebrate_fanfare_sting         265.8  535.2   12.4    11    76    12     1
celebrate_coin_burst              0.5  402.2   21.7     1    53    41     5
celebrate_level_up              627.7  826.0   15.9    11    67    19     3
ui_click_soft                     1.2   17.3   22.0     7    67    25     2
ui_click_hard                     0.5   33.1   20.2    20    62    17     2
ui_hover_tick                     1.3   12.6   24.8     2    47    45     6
ui_coin_pickup                    1.0  251.0   16.6     3    69    24     3
ui_purchase                      91.9  159.4   18.5     5    70    23     2
ui_error_buzz                    16.8  217.5   17.0    61    36     2     0
ui_level_up_short               123.5  186.9   15.8     4    71    23     1
ui_menu_whoosh                   34.3  186.7   17.2    26    59    13     2
amb_wind_ruined_city            656.2 4133.1   16.1    76    22     2     0
amb_distant_battle              942.2 4540.5   16.0    91     9     0     0
amb_zombie_moans_far           1292.7 5254.5   14.6    75    23     2     0
amb_underground_hum            2650.7 2938.3   15.3    67    28     5     0
amb_fire_crackle                851.6 2212.2   22.8    79    19     2     0
```

## Automatic checks

- [x] no NaN / infinite samples
- [x] peak at or below 0 dBFS (limiter + normalisation)
- [x] loop seam below 0.05
- [x] unique ids (no duplicates)

