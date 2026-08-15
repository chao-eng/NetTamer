import { onMounted, ref } from 'vue';
import { Direction, DIRECTION_OPTIONS } from '@/types';
import { useAlertStore } from '@/stores/alertStore';
import { toast } from '@/components/ui/toast';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Button } from '@/components/ui/button';
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table';
import { formatSpeed } from '@/composables/useFormatters';
const alertStore = useAlertStore();
const name = ref('');
const processName = ref('');
const thresholdKb = ref(512);
const direction = ref(Direction.Upload);
const cooldown = ref(30);
async function createRule() {
    if (!processName.value.trim()) {
        toast('请填写进程名', 'error');
        return;
    }
    const rule = {
        id: `R_${processName.value}_${Date.now()}`,
        name: name.value.trim() || `预警-${processName.value}`,
        processName: processName.value.trim(),
        threshold: Math.round(Number(thresholdKb.value) * 1024),
        direction: Number(direction.value),
        cooldownSec: Number(cooldown.value),
        enabled: true,
        createdAt: Math.floor(Date.now() / 1000),
    };
    await alertStore.createRule(rule);
    await alertStore.loadRules();
    toast('已创建预警规则', 'success');
    name.value = '';
    processName.value = '';
}
async function removeRule(id) {
    await alertStore.deleteRule(id);
    toast('已删除规则', 'success');
}
async function toggleEnabled(rule) {
    await alertStore.updateRule({ ...rule, enabled: !rule.enabled });
}
onMounted(async () => {
    await alertStore.loadRules();
    await alertStore.loadHistory();
});
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-4" },
});
const __VLS_0 = {}.Card;
/** @type {[typeof __VLS_components.Card, typeof __VLS_components.Card, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({}));
const __VLS_2 = __VLS_1({}, ...__VLS_functionalComponentArgsRest(__VLS_1));
__VLS_3.slots.default;
const __VLS_4 = {}.CardHeader;
/** @type {[typeof __VLS_components.CardHeader, typeof __VLS_components.CardHeader, ]} */ ;
// @ts-ignore
const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({}));
const __VLS_6 = __VLS_5({}, ...__VLS_functionalComponentArgsRest(__VLS_5));
__VLS_7.slots.default;
const __VLS_8 = {}.CardTitle;
/** @type {[typeof __VLS_components.CardTitle, typeof __VLS_components.CardTitle, ]} */ ;
// @ts-ignore
const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({}));
const __VLS_10 = __VLS_9({}, ...__VLS_functionalComponentArgsRest(__VLS_9));
__VLS_11.slots.default;
var __VLS_11;
const __VLS_12 = {}.CardDescription;
/** @type {[typeof __VLS_components.CardDescription, typeof __VLS_components.CardDescription, ]} */ ;
// @ts-ignore
const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({}));
const __VLS_14 = __VLS_13({}, ...__VLS_functionalComponentArgsRest(__VLS_13));
__VLS_15.slots.default;
var __VLS_15;
var __VLS_7;
const __VLS_16 = {}.CardContent;
/** @type {[typeof __VLS_components.CardContent, typeof __VLS_components.CardContent, ]} */ ;
// @ts-ignore
const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({
    ...{ class: "flex flex-col gap-3" },
}));
const __VLS_18 = __VLS_17({
    ...{ class: "flex flex-col gap-3" },
}, ...__VLS_functionalComponentArgsRest(__VLS_17));
__VLS_19.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "grid grid-cols-1 gap-3 md:grid-cols-2" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_20 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_21 = __VLS_asFunctionalComponent(__VLS_20, new __VLS_20({
    for: "rname",
}));
const __VLS_22 = __VLS_21({
    for: "rname",
}, ...__VLS_functionalComponentArgsRest(__VLS_21));
__VLS_23.slots.default;
var __VLS_23;
const __VLS_24 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_25 = __VLS_asFunctionalComponent(__VLS_24, new __VLS_24({
    id: "rname",
    modelValue: (__VLS_ctx.name),
    placeholder: "可选",
    ...{ class: "mt-1" },
}));
const __VLS_26 = __VLS_25({
    id: "rname",
    modelValue: (__VLS_ctx.name),
    placeholder: "可选",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_25));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_28 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_29 = __VLS_asFunctionalComponent(__VLS_28, new __VLS_28({
    for: "rpname",
}));
const __VLS_30 = __VLS_29({
    for: "rpname",
}, ...__VLS_functionalComponentArgsRest(__VLS_29));
__VLS_31.slots.default;
var __VLS_31;
const __VLS_32 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_33 = __VLS_asFunctionalComponent(__VLS_32, new __VLS_32({
    id: "rpname",
    modelValue: (__VLS_ctx.processName),
    placeholder: "chrome.exe",
    ...{ class: "mt-1" },
}));
const __VLS_34 = __VLS_33({
    id: "rpname",
    modelValue: (__VLS_ctx.processName),
    placeholder: "chrome.exe",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_33));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_36 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_37 = __VLS_asFunctionalComponent(__VLS_36, new __VLS_36({
    for: "rthr",
}));
const __VLS_38 = __VLS_37({
    for: "rthr",
}, ...__VLS_functionalComponentArgsRest(__VLS_37));
__VLS_39.slots.default;
var __VLS_39;
const __VLS_40 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_41 = __VLS_asFunctionalComponent(__VLS_40, new __VLS_40({
    id: "rthr",
    modelValue: (__VLS_ctx.thresholdKb),
    type: "number",
    ...{ class: "mt-1" },
}));
const __VLS_42 = __VLS_41({
    id: "rthr",
    modelValue: (__VLS_ctx.thresholdKb),
    type: "number",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_41));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_44 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_45 = __VLS_asFunctionalComponent(__VLS_44, new __VLS_44({
    for: "rdir",
}));
const __VLS_46 = __VLS_45({
    for: "rdir",
}, ...__VLS_functionalComponentArgsRest(__VLS_45));
__VLS_47.slots.default;
var __VLS_47;
const __VLS_48 = {}.Select;
/** @type {[typeof __VLS_components.Select, ]} */ ;
// @ts-ignore
const __VLS_49 = __VLS_asFunctionalComponent(__VLS_48, new __VLS_48({
    id: "rdir",
    modelValue: (__VLS_ctx.direction),
    options: (__VLS_ctx.DIRECTION_OPTIONS),
    ...{ class: "mt-1" },
}));
const __VLS_50 = __VLS_49({
    id: "rdir",
    modelValue: (__VLS_ctx.direction),
    options: (__VLS_ctx.DIRECTION_OPTIONS),
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_49));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_52 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_53 = __VLS_asFunctionalComponent(__VLS_52, new __VLS_52({
    for: "rcd",
}));
const __VLS_54 = __VLS_53({
    for: "rcd",
}, ...__VLS_functionalComponentArgsRest(__VLS_53));
__VLS_55.slots.default;
var __VLS_55;
const __VLS_56 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_57 = __VLS_asFunctionalComponent(__VLS_56, new __VLS_56({
    id: "rcd",
    modelValue: (__VLS_ctx.cooldown),
    type: "number",
    ...{ class: "mt-1" },
}));
const __VLS_58 = __VLS_57({
    id: "rcd",
    modelValue: (__VLS_ctx.cooldown),
    type: "number",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_57));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex justify-end" },
});
const __VLS_60 = {}.Button;
/** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
// @ts-ignore
const __VLS_61 = __VLS_asFunctionalComponent(__VLS_60, new __VLS_60({
    ...{ 'onClick': {} },
}));
const __VLS_62 = __VLS_61({
    ...{ 'onClick': {} },
}, ...__VLS_functionalComponentArgsRest(__VLS_61));
let __VLS_64;
let __VLS_65;
let __VLS_66;
const __VLS_67 = {
    onClick: (__VLS_ctx.createRule)
};
__VLS_63.slots.default;
var __VLS_63;
var __VLS_19;
var __VLS_3;
const __VLS_68 = {}.Card;
/** @type {[typeof __VLS_components.Card, typeof __VLS_components.Card, ]} */ ;
// @ts-ignore
const __VLS_69 = __VLS_asFunctionalComponent(__VLS_68, new __VLS_68({}));
const __VLS_70 = __VLS_69({}, ...__VLS_functionalComponentArgsRest(__VLS_69));
__VLS_71.slots.default;
const __VLS_72 = {}.CardHeader;
/** @type {[typeof __VLS_components.CardHeader, typeof __VLS_components.CardHeader, ]} */ ;
// @ts-ignore
const __VLS_73 = __VLS_asFunctionalComponent(__VLS_72, new __VLS_72({}));
const __VLS_74 = __VLS_73({}, ...__VLS_functionalComponentArgsRest(__VLS_73));
__VLS_75.slots.default;
const __VLS_76 = {}.CardTitle;
/** @type {[typeof __VLS_components.CardTitle, typeof __VLS_components.CardTitle, ]} */ ;
// @ts-ignore
const __VLS_77 = __VLS_asFunctionalComponent(__VLS_76, new __VLS_76({}));
const __VLS_78 = __VLS_77({}, ...__VLS_functionalComponentArgsRest(__VLS_77));
__VLS_79.slots.default;
(__VLS_ctx.alertStore.rules.length);
var __VLS_79;
var __VLS_75;
const __VLS_80 = {}.CardContent;
/** @type {[typeof __VLS_components.CardContent, typeof __VLS_components.CardContent, ]} */ ;
// @ts-ignore
const __VLS_81 = __VLS_asFunctionalComponent(__VLS_80, new __VLS_80({}));
const __VLS_82 = __VLS_81({}, ...__VLS_functionalComponentArgsRest(__VLS_81));
__VLS_83.slots.default;
const __VLS_84 = {}.Table;
/** @type {[typeof __VLS_components.Table, typeof __VLS_components.Table, ]} */ ;
// @ts-ignore
const __VLS_85 = __VLS_asFunctionalComponent(__VLS_84, new __VLS_84({}));
const __VLS_86 = __VLS_85({}, ...__VLS_functionalComponentArgsRest(__VLS_85));
__VLS_87.slots.default;
const __VLS_88 = {}.TableHeader;
/** @type {[typeof __VLS_components.TableHeader, typeof __VLS_components.TableHeader, ]} */ ;
// @ts-ignore
const __VLS_89 = __VLS_asFunctionalComponent(__VLS_88, new __VLS_88({}));
const __VLS_90 = __VLS_89({}, ...__VLS_functionalComponentArgsRest(__VLS_89));
__VLS_91.slots.default;
const __VLS_92 = {}.TableRow;
/** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
// @ts-ignore
const __VLS_93 = __VLS_asFunctionalComponent(__VLS_92, new __VLS_92({}));
const __VLS_94 = __VLS_93({}, ...__VLS_functionalComponentArgsRest(__VLS_93));
__VLS_95.slots.default;
const __VLS_96 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_97 = __VLS_asFunctionalComponent(__VLS_96, new __VLS_96({}));
const __VLS_98 = __VLS_97({}, ...__VLS_functionalComponentArgsRest(__VLS_97));
__VLS_99.slots.default;
var __VLS_99;
const __VLS_100 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_101 = __VLS_asFunctionalComponent(__VLS_100, new __VLS_100({}));
const __VLS_102 = __VLS_101({}, ...__VLS_functionalComponentArgsRest(__VLS_101));
__VLS_103.slots.default;
var __VLS_103;
const __VLS_104 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_105 = __VLS_asFunctionalComponent(__VLS_104, new __VLS_104({
    ...{ class: "text-right" },
}));
const __VLS_106 = __VLS_105({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_105));
__VLS_107.slots.default;
var __VLS_107;
const __VLS_108 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_109 = __VLS_asFunctionalComponent(__VLS_108, new __VLS_108({
    ...{ class: "text-right" },
}));
const __VLS_110 = __VLS_109({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_109));
__VLS_111.slots.default;
var __VLS_111;
const __VLS_112 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_113 = __VLS_asFunctionalComponent(__VLS_112, new __VLS_112({
    ...{ class: "text-right" },
}));
const __VLS_114 = __VLS_113({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_113));
__VLS_115.slots.default;
var __VLS_115;
var __VLS_95;
var __VLS_91;
const __VLS_116 = {}.TableBody;
/** @type {[typeof __VLS_components.TableBody, typeof __VLS_components.TableBody, ]} */ ;
// @ts-ignore
const __VLS_117 = __VLS_asFunctionalComponent(__VLS_116, new __VLS_116({}));
const __VLS_118 = __VLS_117({}, ...__VLS_functionalComponentArgsRest(__VLS_117));
__VLS_119.slots.default;
for (const [r] of __VLS_getVForSourceType((__VLS_ctx.alertStore.rules))) {
    const __VLS_120 = {}.TableRow;
    /** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
    // @ts-ignore
    const __VLS_121 = __VLS_asFunctionalComponent(__VLS_120, new __VLS_120({
        key: (r.id),
    }));
    const __VLS_122 = __VLS_121({
        key: (r.id),
    }, ...__VLS_functionalComponentArgsRest(__VLS_121));
    __VLS_123.slots.default;
    const __VLS_124 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_125 = __VLS_asFunctionalComponent(__VLS_124, new __VLS_124({}));
    const __VLS_126 = __VLS_125({}, ...__VLS_functionalComponentArgsRest(__VLS_125));
    __VLS_127.slots.default;
    (r.name);
    var __VLS_127;
    const __VLS_128 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_129 = __VLS_asFunctionalComponent(__VLS_128, new __VLS_128({}));
    const __VLS_130 = __VLS_129({}, ...__VLS_functionalComponentArgsRest(__VLS_129));
    __VLS_131.slots.default;
    (r.processName);
    var __VLS_131;
    const __VLS_132 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_133 = __VLS_asFunctionalComponent(__VLS_132, new __VLS_132({
        ...{ class: "text-right" },
    }));
    const __VLS_134 = __VLS_133({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_133));
    __VLS_135.slots.default;
    (__VLS_ctx.formatSpeed(r.threshold));
    var __VLS_135;
    const __VLS_136 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_137 = __VLS_asFunctionalComponent(__VLS_136, new __VLS_136({
        ...{ class: "text-right" },
    }));
    const __VLS_138 = __VLS_137({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_137));
    __VLS_139.slots.default;
    const __VLS_140 = {}.Switch;
    /** @type {[typeof __VLS_components.Switch, ]} */ ;
    // @ts-ignore
    const __VLS_141 = __VLS_asFunctionalComponent(__VLS_140, new __VLS_140({
        ...{ 'onUpdate:modelValue': {} },
        modelValue: (r.enabled),
    }));
    const __VLS_142 = __VLS_141({
        ...{ 'onUpdate:modelValue': {} },
        modelValue: (r.enabled),
    }, ...__VLS_functionalComponentArgsRest(__VLS_141));
    let __VLS_144;
    let __VLS_145;
    let __VLS_146;
    const __VLS_147 = {
        'onUpdate:modelValue': (...[$event]) => {
            __VLS_ctx.toggleEnabled(r);
        }
    };
    var __VLS_143;
    var __VLS_139;
    const __VLS_148 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_149 = __VLS_asFunctionalComponent(__VLS_148, new __VLS_148({
        ...{ class: "text-right" },
    }));
    const __VLS_150 = __VLS_149({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_149));
    __VLS_151.slots.default;
    const __VLS_152 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_153 = __VLS_asFunctionalComponent(__VLS_152, new __VLS_152({
        ...{ 'onClick': {} },
        size: "sm",
        variant: "ghost",
    }));
    const __VLS_154 = __VLS_153({
        ...{ 'onClick': {} },
        size: "sm",
        variant: "ghost",
    }, ...__VLS_functionalComponentArgsRest(__VLS_153));
    let __VLS_156;
    let __VLS_157;
    let __VLS_158;
    const __VLS_159 = {
        onClick: (...[$event]) => {
            __VLS_ctx.removeRule(r.id);
        }
    };
    __VLS_155.slots.default;
    var __VLS_155;
    var __VLS_151;
    var __VLS_123;
}
if (__VLS_ctx.alertStore.rules.length === 0) {
    const __VLS_160 = {}.TableRow;
    /** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
    // @ts-ignore
    const __VLS_161 = __VLS_asFunctionalComponent(__VLS_160, new __VLS_160({}));
    const __VLS_162 = __VLS_161({}, ...__VLS_functionalComponentArgsRest(__VLS_161));
    __VLS_163.slots.default;
    const __VLS_164 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_165 = __VLS_asFunctionalComponent(__VLS_164, new __VLS_164({
        colspan: "5",
        ...{ class: "text-center text-muted-foreground" },
    }));
    const __VLS_166 = __VLS_165({
        colspan: "5",
        ...{ class: "text-center text-muted-foreground" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_165));
    __VLS_167.slots.default;
    var __VLS_167;
    var __VLS_163;
}
var __VLS_119;
var __VLS_87;
var __VLS_83;
var __VLS_71;
const __VLS_168 = {}.Card;
/** @type {[typeof __VLS_components.Card, typeof __VLS_components.Card, ]} */ ;
// @ts-ignore
const __VLS_169 = __VLS_asFunctionalComponent(__VLS_168, new __VLS_168({}));
const __VLS_170 = __VLS_169({}, ...__VLS_functionalComponentArgsRest(__VLS_169));
__VLS_171.slots.default;
const __VLS_172 = {}.CardHeader;
/** @type {[typeof __VLS_components.CardHeader, typeof __VLS_components.CardHeader, ]} */ ;
// @ts-ignore
const __VLS_173 = __VLS_asFunctionalComponent(__VLS_172, new __VLS_172({}));
const __VLS_174 = __VLS_173({}, ...__VLS_functionalComponentArgsRest(__VLS_173));
__VLS_175.slots.default;
const __VLS_176 = {}.CardTitle;
/** @type {[typeof __VLS_components.CardTitle, typeof __VLS_components.CardTitle, ]} */ ;
// @ts-ignore
const __VLS_177 = __VLS_asFunctionalComponent(__VLS_176, new __VLS_176({}));
const __VLS_178 = __VLS_177({}, ...__VLS_functionalComponentArgsRest(__VLS_177));
__VLS_179.slots.default;
(__VLS_ctx.alertStore.history.length);
var __VLS_179;
var __VLS_175;
const __VLS_180 = {}.CardContent;
/** @type {[typeof __VLS_components.CardContent, typeof __VLS_components.CardContent, ]} */ ;
// @ts-ignore
const __VLS_181 = __VLS_asFunctionalComponent(__VLS_180, new __VLS_180({}));
const __VLS_182 = __VLS_181({}, ...__VLS_functionalComponentArgsRest(__VLS_181));
__VLS_183.slots.default;
const __VLS_184 = {}.Table;
/** @type {[typeof __VLS_components.Table, typeof __VLS_components.Table, ]} */ ;
// @ts-ignore
const __VLS_185 = __VLS_asFunctionalComponent(__VLS_184, new __VLS_184({}));
const __VLS_186 = __VLS_185({}, ...__VLS_functionalComponentArgsRest(__VLS_185));
__VLS_187.slots.default;
const __VLS_188 = {}.TableHeader;
/** @type {[typeof __VLS_components.TableHeader, typeof __VLS_components.TableHeader, ]} */ ;
// @ts-ignore
const __VLS_189 = __VLS_asFunctionalComponent(__VLS_188, new __VLS_188({}));
const __VLS_190 = __VLS_189({}, ...__VLS_functionalComponentArgsRest(__VLS_189));
__VLS_191.slots.default;
const __VLS_192 = {}.TableRow;
/** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
// @ts-ignore
const __VLS_193 = __VLS_asFunctionalComponent(__VLS_192, new __VLS_192({}));
const __VLS_194 = __VLS_193({}, ...__VLS_functionalComponentArgsRest(__VLS_193));
__VLS_195.slots.default;
const __VLS_196 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_197 = __VLS_asFunctionalComponent(__VLS_196, new __VLS_196({}));
const __VLS_198 = __VLS_197({}, ...__VLS_functionalComponentArgsRest(__VLS_197));
__VLS_199.slots.default;
var __VLS_199;
const __VLS_200 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_201 = __VLS_asFunctionalComponent(__VLS_200, new __VLS_200({
    ...{ class: "text-right" },
}));
const __VLS_202 = __VLS_201({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_201));
__VLS_203.slots.default;
var __VLS_203;
const __VLS_204 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_205 = __VLS_asFunctionalComponent(__VLS_204, new __VLS_204({
    ...{ class: "text-right" },
}));
const __VLS_206 = __VLS_205({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_205));
__VLS_207.slots.default;
var __VLS_207;
const __VLS_208 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_209 = __VLS_asFunctionalComponent(__VLS_208, new __VLS_208({
    ...{ class: "text-right" },
}));
const __VLS_210 = __VLS_209({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_209));
__VLS_211.slots.default;
var __VLS_211;
var __VLS_195;
var __VLS_191;
const __VLS_212 = {}.TableBody;
/** @type {[typeof __VLS_components.TableBody, typeof __VLS_components.TableBody, ]} */ ;
// @ts-ignore
const __VLS_213 = __VLS_asFunctionalComponent(__VLS_212, new __VLS_212({}));
const __VLS_214 = __VLS_213({}, ...__VLS_functionalComponentArgsRest(__VLS_213));
__VLS_215.slots.default;
for (const [h] of __VLS_getVForSourceType((__VLS_ctx.alertStore.history))) {
    const __VLS_216 = {}.TableRow;
    /** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
    // @ts-ignore
    const __VLS_217 = __VLS_asFunctionalComponent(__VLS_216, new __VLS_216({
        key: (h.id),
    }));
    const __VLS_218 = __VLS_217({
        key: (h.id),
    }, ...__VLS_functionalComponentArgsRest(__VLS_217));
    __VLS_219.slots.default;
    const __VLS_220 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_221 = __VLS_asFunctionalComponent(__VLS_220, new __VLS_220({}));
    const __VLS_222 = __VLS_221({}, ...__VLS_functionalComponentArgsRest(__VLS_221));
    __VLS_223.slots.default;
    (h.processName);
    var __VLS_223;
    const __VLS_224 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_225 = __VLS_asFunctionalComponent(__VLS_224, new __VLS_224({
        ...{ class: "text-right" },
    }));
    const __VLS_226 = __VLS_225({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_225));
    __VLS_227.slots.default;
    (__VLS_ctx.formatSpeed(h.currentRate));
    var __VLS_227;
    const __VLS_228 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_229 = __VLS_asFunctionalComponent(__VLS_228, new __VLS_228({
        ...{ class: "text-right" },
    }));
    const __VLS_230 = __VLS_229({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_229));
    __VLS_231.slots.default;
    (__VLS_ctx.formatSpeed(h.threshold));
    var __VLS_231;
    const __VLS_232 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_233 = __VLS_asFunctionalComponent(__VLS_232, new __VLS_232({
        ...{ class: "text-right text-muted-foreground" },
    }));
    const __VLS_234 = __VLS_233({
        ...{ class: "text-right text-muted-foreground" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_233));
    __VLS_235.slots.default;
    (new Date(h.triggeredAt * 1000).toLocaleTimeString());
    var __VLS_235;
    var __VLS_219;
}
if (__VLS_ctx.alertStore.history.length === 0) {
    const __VLS_236 = {}.TableRow;
    /** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
    // @ts-ignore
    const __VLS_237 = __VLS_asFunctionalComponent(__VLS_236, new __VLS_236({}));
    const __VLS_238 = __VLS_237({}, ...__VLS_functionalComponentArgsRest(__VLS_237));
    __VLS_239.slots.default;
    const __VLS_240 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_241 = __VLS_asFunctionalComponent(__VLS_240, new __VLS_240({
        colspan: "4",
        ...{ class: "text-center text-muted-foreground" },
    }));
    const __VLS_242 = __VLS_241({
        colspan: "4",
        ...{ class: "text-center text-muted-foreground" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_241));
    __VLS_243.slots.default;
    var __VLS_243;
    var __VLS_239;
}
var __VLS_215;
var __VLS_187;
var __VLS_183;
var __VLS_171;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['grid']} */ ;
/** @type {__VLS_StyleScopedClasses['grid-cols-1']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['md:grid-cols-2']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-end']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-muted-foreground']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-muted-foreground']} */ ;
/** @type {__VLS_StyleScopedClasses['text-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-muted-foreground']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            DIRECTION_OPTIONS: DIRECTION_OPTIONS,
            Card: Card,
            CardHeader: CardHeader,
            CardTitle: CardTitle,
            CardDescription: CardDescription,
            CardContent: CardContent,
            Input: Input,
            Label: Label,
            Select: Select,
            Switch: Switch,
            Button: Button,
            Table: Table,
            TableHeader: TableHeader,
            TableBody: TableBody,
            TableRow: TableRow,
            TableHead: TableHead,
            TableCell: TableCell,
            formatSpeed: formatSpeed,
            alertStore: alertStore,
            name: name,
            processName: processName,
            thresholdKb: thresholdKb,
            direction: direction,
            cooldown: cooldown,
            createRule: createRule,
            removeRule: removeRule,
            toggleEnabled: toggleEnabled,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
