import { onMounted, onBeforeUnmount, ref, computed } from 'vue';
import { Direction, DIRECTION_OPTIONS } from '@/types';
import { useProcessStore } from '@/stores/processStore';
import { useThrottleStore } from '@/stores/throttleStore';
import { useAlertStore } from '@/stores/alertStore';
import { toast } from '@/components/ui/toast';
import { Card, CardContent, } from '@/components/ui/card';
import ProcessIcon from '@/components/common/ProcessIcon.vue';
import SpeedBadge from '@/components/common/SpeedBadge.vue';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell, } from '@/components/ui/table';
import { Dialog } from '@/components/ui/dialog';
const processStore = useProcessStore();
const throttleStore = useThrottleStore();
const alertStore = useAlertStore();
const throttleOpen = ref(false);
const alertOpen = ref(false);
const selected = ref(null);
// 限速表单
const kbps = ref(512);
const limitUpload = ref(true);
const limitDownload = ref(true);
// 预警表单
const thresholdKb = ref(512);
const direction = ref(Direction.Upload);
const cooldown = ref(30);
const sorted = computed(() => processStore.sortedProcesses);
function sortBy(field) {
    processStore.setSort(field);
}
function openThrottle(p) {
    selected.value = p;
    throttleOpen.value = true;
}
function openAlert(p) {
    selected.value = p;
    alertOpen.value = true;
}
async function applyThrottle() {
    if (!selected.value)
        return;
    const policy = {
        id: `NT_${selected.value.name}_${Date.now()}`,
        name: `NT_${selected.value.name}`,
        processName: selected.value.name,
        rateLimitBps: Math.round(kbps.value * 1024 * 8),
        limitUpload: limitUpload.value,
        limitDownload: limitDownload.value,
        active: true,
        createdAt: Math.floor(Date.now() / 1000),
    };
    await throttleStore.apply(policy);
    await throttleStore.load();
    toast('已应用限速策略', 'success');
    throttleOpen.value = false;
}
async function createAlert() {
    if (!selected.value)
        return;
    const rule = {
        id: `R_${selected.value.name}_${Date.now()}`,
        name: `预警-${selected.value.name}`,
        processName: selected.value.name,
        threshold: Math.round(Number(thresholdKb.value) * 1024),
        direction: Number(direction.value),
        cooldownSec: Number(cooldown.value),
        enabled: true,
        createdAt: Math.floor(Date.now() / 1000),
    };
    await alertStore.createRule(rule);
    await alertStore.loadRules();
    toast('已创建预警规则', 'success');
    alertOpen.value = false;
}
let unlisten = [];
onMounted(async () => {
    await processStore.fetchList();
    unlisten = await processStore.bindEvents();
    if (!processStore.isMonitoring) {
        await processStore.start();
    }
});
onBeforeUnmount(() => {
    processStore.searchQuery = '';
    unlisten.forEach((fn) => fn());
});
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-4" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-3" },
});
const __VLS_0 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
    modelValue: (__VLS_ctx.processStore.searchQuery),
    placeholder: "搜索进程名...",
    ...{ class: "max-w-sm" },
}));
const __VLS_2 = __VLS_1({
    modelValue: (__VLS_ctx.processStore.searchQuery),
    placeholder: "搜索进程名...",
    ...{ class: "max-w-sm" },
}, ...__VLS_functionalComponentArgsRest(__VLS_1));
const __VLS_4 = {}.Badge;
/** @type {[typeof __VLS_components.Badge, typeof __VLS_components.Badge, ]} */ ;
// @ts-ignore
const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
    variant: "secondary",
}));
const __VLS_6 = __VLS_5({
    variant: "secondary",
}, ...__VLS_functionalComponentArgsRest(__VLS_5));
__VLS_7.slots.default;
(__VLS_ctx.sorted.length);
var __VLS_7;
const __VLS_8 = {}.Card;
/** @type {[typeof __VLS_components.Card, typeof __VLS_components.Card, ]} */ ;
// @ts-ignore
const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({}));
const __VLS_10 = __VLS_9({}, ...__VLS_functionalComponentArgsRest(__VLS_9));
__VLS_11.slots.default;
const __VLS_12 = {}.CardContent;
/** @type {[typeof __VLS_components.CardContent, typeof __VLS_components.CardContent, ]} */ ;
// @ts-ignore
const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
    ...{ class: "pt-6" },
}));
const __VLS_14 = __VLS_13({
    ...{ class: "pt-6" },
}, ...__VLS_functionalComponentArgsRest(__VLS_13));
__VLS_15.slots.default;
const __VLS_16 = {}.Table;
/** @type {[typeof __VLS_components.Table, typeof __VLS_components.Table, ]} */ ;
// @ts-ignore
const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({}));
const __VLS_18 = __VLS_17({}, ...__VLS_functionalComponentArgsRest(__VLS_17));
__VLS_19.slots.default;
const __VLS_20 = {}.TableHeader;
/** @type {[typeof __VLS_components.TableHeader, typeof __VLS_components.TableHeader, ]} */ ;
// @ts-ignore
const __VLS_21 = __VLS_asFunctionalComponent(__VLS_20, new __VLS_20({}));
const __VLS_22 = __VLS_21({}, ...__VLS_functionalComponentArgsRest(__VLS_21));
__VLS_23.slots.default;
const __VLS_24 = {}.TableRow;
/** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
// @ts-ignore
const __VLS_25 = __VLS_asFunctionalComponent(__VLS_24, new __VLS_24({}));
const __VLS_26 = __VLS_25({}, ...__VLS_functionalComponentArgsRest(__VLS_25));
__VLS_27.slots.default;
const __VLS_28 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_29 = __VLS_asFunctionalComponent(__VLS_28, new __VLS_28({}));
const __VLS_30 = __VLS_29({}, ...__VLS_functionalComponentArgsRest(__VLS_29));
__VLS_31.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (...[$event]) => {
            __VLS_ctx.sortBy('name');
        } },
    ...{ class: "font-medium hover:underline" },
});
var __VLS_31;
const __VLS_32 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_33 = __VLS_asFunctionalComponent(__VLS_32, new __VLS_32({}));
const __VLS_34 = __VLS_33({}, ...__VLS_functionalComponentArgsRest(__VLS_33));
__VLS_35.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (...[$event]) => {
            __VLS_ctx.sortBy('pid');
        } },
    ...{ class: "font-medium hover:underline" },
});
var __VLS_35;
const __VLS_36 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_37 = __VLS_asFunctionalComponent(__VLS_36, new __VLS_36({
    ...{ class: "text-right" },
}));
const __VLS_38 = __VLS_37({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_37));
__VLS_39.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (...[$event]) => {
            __VLS_ctx.sortBy('uploadRate');
        } },
    ...{ class: "font-medium hover:underline" },
});
var __VLS_39;
const __VLS_40 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_41 = __VLS_asFunctionalComponent(__VLS_40, new __VLS_40({
    ...{ class: "text-right" },
}));
const __VLS_42 = __VLS_41({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_41));
__VLS_43.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (...[$event]) => {
            __VLS_ctx.sortBy('downloadRate');
        } },
    ...{ class: "font-medium hover:underline" },
});
var __VLS_43;
const __VLS_44 = {}.TableHead;
/** @type {[typeof __VLS_components.TableHead, typeof __VLS_components.TableHead, ]} */ ;
// @ts-ignore
const __VLS_45 = __VLS_asFunctionalComponent(__VLS_44, new __VLS_44({
    ...{ class: "text-right" },
}));
const __VLS_46 = __VLS_45({
    ...{ class: "text-right" },
}, ...__VLS_functionalComponentArgsRest(__VLS_45));
__VLS_47.slots.default;
var __VLS_47;
var __VLS_27;
var __VLS_23;
const __VLS_48 = {}.TableBody;
/** @type {[typeof __VLS_components.TableBody, typeof __VLS_components.TableBody, ]} */ ;
// @ts-ignore
const __VLS_49 = __VLS_asFunctionalComponent(__VLS_48, new __VLS_48({}));
const __VLS_50 = __VLS_49({}, ...__VLS_functionalComponentArgsRest(__VLS_49));
__VLS_51.slots.default;
for (const [p] of __VLS_getVForSourceType((__VLS_ctx.sorted))) {
    const __VLS_52 = {}.TableRow;
    /** @type {[typeof __VLS_components.TableRow, typeof __VLS_components.TableRow, ]} */ ;
    // @ts-ignore
    const __VLS_53 = __VLS_asFunctionalComponent(__VLS_52, new __VLS_52({
        key: (p.pid),
    }));
    const __VLS_54 = __VLS_53({
        key: (p.pid),
    }, ...__VLS_functionalComponentArgsRest(__VLS_53));
    __VLS_55.slots.default;
    const __VLS_56 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_57 = __VLS_asFunctionalComponent(__VLS_56, new __VLS_56({}));
    const __VLS_58 = __VLS_57({}, ...__VLS_functionalComponentArgsRest(__VLS_57));
    __VLS_59.slots.default;
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex items-center gap-2" },
    });
    /** @type {[typeof ProcessIcon, ]} */ ;
    // @ts-ignore
    const __VLS_60 = __VLS_asFunctionalComponent(ProcessIcon, new ProcessIcon({
        iconB64: (p.iconB64),
        name: (p.name),
    }));
    const __VLS_61 = __VLS_60({
        iconB64: (p.iconB64),
        name: (p.name),
    }, ...__VLS_functionalComponentArgsRest(__VLS_60));
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "truncate" },
    });
    (p.name);
    var __VLS_59;
    const __VLS_63 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_64 = __VLS_asFunctionalComponent(__VLS_63, new __VLS_63({
        ...{ class: "tabular" },
    }));
    const __VLS_65 = __VLS_64({
        ...{ class: "tabular" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_64));
    __VLS_66.slots.default;
    (p.pid);
    var __VLS_66;
    const __VLS_67 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_68 = __VLS_asFunctionalComponent(__VLS_67, new __VLS_67({
        ...{ class: "text-right" },
    }));
    const __VLS_69 = __VLS_68({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_68));
    __VLS_70.slots.default;
    /** @type {[typeof SpeedBadge, ]} */ ;
    // @ts-ignore
    const __VLS_71 = __VLS_asFunctionalComponent(SpeedBadge, new SpeedBadge({
        rate: (p.uploadRate),
        direction: "up",
    }));
    const __VLS_72 = __VLS_71({
        rate: (p.uploadRate),
        direction: "up",
    }, ...__VLS_functionalComponentArgsRest(__VLS_71));
    var __VLS_70;
    const __VLS_74 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_75 = __VLS_asFunctionalComponent(__VLS_74, new __VLS_74({
        ...{ class: "text-right" },
    }));
    const __VLS_76 = __VLS_75({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_75));
    __VLS_77.slots.default;
    /** @type {[typeof SpeedBadge, ]} */ ;
    // @ts-ignore
    const __VLS_78 = __VLS_asFunctionalComponent(SpeedBadge, new SpeedBadge({
        rate: (p.downloadRate),
        direction: "down",
    }));
    const __VLS_79 = __VLS_78({
        rate: (p.downloadRate),
        direction: "down",
    }, ...__VLS_functionalComponentArgsRest(__VLS_78));
    var __VLS_77;
    const __VLS_81 = {}.TableCell;
    /** @type {[typeof __VLS_components.TableCell, typeof __VLS_components.TableCell, ]} */ ;
    // @ts-ignore
    const __VLS_82 = __VLS_asFunctionalComponent(__VLS_81, new __VLS_81({
        ...{ class: "text-right" },
    }));
    const __VLS_83 = __VLS_82({
        ...{ class: "text-right" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_82));
    __VLS_84.slots.default;
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex justify-end gap-2" },
    });
    const __VLS_85 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_86 = __VLS_asFunctionalComponent(__VLS_85, new __VLS_85({
        ...{ 'onClick': {} },
        size: "sm",
        variant: "outline",
    }));
    const __VLS_87 = __VLS_86({
        ...{ 'onClick': {} },
        size: "sm",
        variant: "outline",
    }, ...__VLS_functionalComponentArgsRest(__VLS_86));
    let __VLS_89;
    let __VLS_90;
    let __VLS_91;
    const __VLS_92 = {
        onClick: (...[$event]) => {
            __VLS_ctx.openThrottle(p);
        }
    };
    __VLS_88.slots.default;
    var __VLS_88;
    const __VLS_93 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_94 = __VLS_asFunctionalComponent(__VLS_93, new __VLS_93({
        ...{ 'onClick': {} },
        size: "sm",
        variant: "outline",
    }));
    const __VLS_95 = __VLS_94({
        ...{ 'onClick': {} },
        size: "sm",
        variant: "outline",
    }, ...__VLS_functionalComponentArgsRest(__VLS_94));
    let __VLS_97;
    let __VLS_98;
    let __VLS_99;
    const __VLS_100 = {
        onClick: (...[$event]) => {
            __VLS_ctx.openAlert(p);
        }
    };
    __VLS_96.slots.default;
    var __VLS_96;
    var __VLS_84;
    var __VLS_55;
}
var __VLS_51;
var __VLS_19;
var __VLS_15;
var __VLS_11;
const __VLS_101 = {}.Dialog;
/** @type {[typeof __VLS_components.Dialog, typeof __VLS_components.Dialog, ]} */ ;
// @ts-ignore
const __VLS_102 = __VLS_asFunctionalComponent(__VLS_101, new __VLS_101({
    open: (__VLS_ctx.throttleOpen),
    title: "进程限速",
}));
const __VLS_103 = __VLS_102({
    open: (__VLS_ctx.throttleOpen),
    title: "进程限速",
}, ...__VLS_functionalComponentArgsRest(__VLS_102));
__VLS_104.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-3" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_105 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_106 = __VLS_asFunctionalComponent(__VLS_105, new __VLS_105({}));
const __VLS_107 = __VLS_106({}, ...__VLS_functionalComponentArgsRest(__VLS_106));
__VLS_108.slots.default;
var __VLS_108;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "mt-1 text-sm font-medium" },
});
(__VLS_ctx.selected?.name);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_109 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_110 = __VLS_asFunctionalComponent(__VLS_109, new __VLS_109({
    for: "kbps",
}));
const __VLS_111 = __VLS_110({
    for: "kbps",
}, ...__VLS_functionalComponentArgsRest(__VLS_110));
__VLS_112.slots.default;
var __VLS_112;
const __VLS_113 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_114 = __VLS_asFunctionalComponent(__VLS_113, new __VLS_113({
    id: "kbps",
    modelValue: (__VLS_ctx.kbps),
    type: "number",
    ...{ class: "mt-1" },
}));
const __VLS_115 = __VLS_114({
    id: "kbps",
    modelValue: (__VLS_ctx.kbps),
    type: "number",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_114));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center justify-between" },
});
const __VLS_117 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_118 = __VLS_asFunctionalComponent(__VLS_117, new __VLS_117({}));
const __VLS_119 = __VLS_118({}, ...__VLS_functionalComponentArgsRest(__VLS_118));
__VLS_120.slots.default;
var __VLS_120;
const __VLS_121 = {}.Switch;
/** @type {[typeof __VLS_components.Switch, ]} */ ;
// @ts-ignore
const __VLS_122 = __VLS_asFunctionalComponent(__VLS_121, new __VLS_121({
    modelValue: (__VLS_ctx.limitUpload),
}));
const __VLS_123 = __VLS_122({
    modelValue: (__VLS_ctx.limitUpload),
}, ...__VLS_functionalComponentArgsRest(__VLS_122));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center justify-between" },
});
const __VLS_125 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_126 = __VLS_asFunctionalComponent(__VLS_125, new __VLS_125({}));
const __VLS_127 = __VLS_126({}, ...__VLS_functionalComponentArgsRest(__VLS_126));
__VLS_128.slots.default;
var __VLS_128;
const __VLS_129 = {}.Switch;
/** @type {[typeof __VLS_components.Switch, ]} */ ;
// @ts-ignore
const __VLS_130 = __VLS_asFunctionalComponent(__VLS_129, new __VLS_129({
    modelValue: (__VLS_ctx.limitDownload),
}));
const __VLS_131 = __VLS_130({
    modelValue: (__VLS_ctx.limitDownload),
}, ...__VLS_functionalComponentArgsRest(__VLS_130));
{
    const { footer: __VLS_thisSlot } = __VLS_104.slots;
    const __VLS_133 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_134 = __VLS_asFunctionalComponent(__VLS_133, new __VLS_133({
        ...{ 'onClick': {} },
        variant: "ghost",
    }));
    const __VLS_135 = __VLS_134({
        ...{ 'onClick': {} },
        variant: "ghost",
    }, ...__VLS_functionalComponentArgsRest(__VLS_134));
    let __VLS_137;
    let __VLS_138;
    let __VLS_139;
    const __VLS_140 = {
        onClick: (...[$event]) => {
            __VLS_ctx.throttleOpen = false;
        }
    };
    __VLS_136.slots.default;
    var __VLS_136;
    const __VLS_141 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_142 = __VLS_asFunctionalComponent(__VLS_141, new __VLS_141({
        ...{ 'onClick': {} },
    }));
    const __VLS_143 = __VLS_142({
        ...{ 'onClick': {} },
    }, ...__VLS_functionalComponentArgsRest(__VLS_142));
    let __VLS_145;
    let __VLS_146;
    let __VLS_147;
    const __VLS_148 = {
        onClick: (__VLS_ctx.applyThrottle)
    };
    __VLS_144.slots.default;
    var __VLS_144;
}
var __VLS_104;
const __VLS_149 = {}.Dialog;
/** @type {[typeof __VLS_components.Dialog, typeof __VLS_components.Dialog, ]} */ ;
// @ts-ignore
const __VLS_150 = __VLS_asFunctionalComponent(__VLS_149, new __VLS_149({
    open: (__VLS_ctx.alertOpen),
    title: "创建预警规则",
}));
const __VLS_151 = __VLS_150({
    open: (__VLS_ctx.alertOpen),
    title: "创建预警规则",
}, ...__VLS_functionalComponentArgsRest(__VLS_150));
__VLS_152.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-3" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_153 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_154 = __VLS_asFunctionalComponent(__VLS_153, new __VLS_153({}));
const __VLS_155 = __VLS_154({}, ...__VLS_functionalComponentArgsRest(__VLS_154));
__VLS_156.slots.default;
var __VLS_156;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "mt-1 text-sm font-medium" },
});
(__VLS_ctx.selected?.name);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_157 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_158 = __VLS_asFunctionalComponent(__VLS_157, new __VLS_157({
    for: "thr",
}));
const __VLS_159 = __VLS_158({
    for: "thr",
}, ...__VLS_functionalComponentArgsRest(__VLS_158));
__VLS_160.slots.default;
var __VLS_160;
const __VLS_161 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_162 = __VLS_asFunctionalComponent(__VLS_161, new __VLS_161({
    id: "thr",
    modelValue: (__VLS_ctx.thresholdKb),
    type: "number",
    ...{ class: "mt-1" },
}));
const __VLS_163 = __VLS_162({
    id: "thr",
    modelValue: (__VLS_ctx.thresholdKb),
    type: "number",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_162));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_165 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_166 = __VLS_asFunctionalComponent(__VLS_165, new __VLS_165({}));
const __VLS_167 = __VLS_166({}, ...__VLS_functionalComponentArgsRest(__VLS_166));
__VLS_168.slots.default;
var __VLS_168;
const __VLS_169 = {}.Select;
/** @type {[typeof __VLS_components.Select, ]} */ ;
// @ts-ignore
const __VLS_170 = __VLS_asFunctionalComponent(__VLS_169, new __VLS_169({
    modelValue: (__VLS_ctx.direction),
    options: (__VLS_ctx.DIRECTION_OPTIONS),
    ...{ class: "mt-1" },
}));
const __VLS_171 = __VLS_170({
    modelValue: (__VLS_ctx.direction),
    options: (__VLS_ctx.DIRECTION_OPTIONS),
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_170));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
const __VLS_173 = {}.Label;
/** @type {[typeof __VLS_components.Label, typeof __VLS_components.Label, ]} */ ;
// @ts-ignore
const __VLS_174 = __VLS_asFunctionalComponent(__VLS_173, new __VLS_173({
    for: "cd",
}));
const __VLS_175 = __VLS_174({
    for: "cd",
}, ...__VLS_functionalComponentArgsRest(__VLS_174));
__VLS_176.slots.default;
var __VLS_176;
const __VLS_177 = {}.Input;
/** @type {[typeof __VLS_components.Input, ]} */ ;
// @ts-ignore
const __VLS_178 = __VLS_asFunctionalComponent(__VLS_177, new __VLS_177({
    id: "cd",
    modelValue: (__VLS_ctx.cooldown),
    type: "number",
    ...{ class: "mt-1" },
}));
const __VLS_179 = __VLS_178({
    id: "cd",
    modelValue: (__VLS_ctx.cooldown),
    type: "number",
    ...{ class: "mt-1" },
}, ...__VLS_functionalComponentArgsRest(__VLS_178));
{
    const { footer: __VLS_thisSlot } = __VLS_152.slots;
    const __VLS_181 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_182 = __VLS_asFunctionalComponent(__VLS_181, new __VLS_181({
        ...{ 'onClick': {} },
        variant: "ghost",
    }));
    const __VLS_183 = __VLS_182({
        ...{ 'onClick': {} },
        variant: "ghost",
    }, ...__VLS_functionalComponentArgsRest(__VLS_182));
    let __VLS_185;
    let __VLS_186;
    let __VLS_187;
    const __VLS_188 = {
        onClick: (...[$event]) => {
            __VLS_ctx.alertOpen = false;
        }
    };
    __VLS_184.slots.default;
    var __VLS_184;
    const __VLS_189 = {}.Button;
    /** @type {[typeof __VLS_components.Button, typeof __VLS_components.Button, ]} */ ;
    // @ts-ignore
    const __VLS_190 = __VLS_asFunctionalComponent(__VLS_189, new __VLS_189({
        ...{ 'onClick': {} },
    }));
    const __VLS_191 = __VLS_190({
        ...{ 'onClick': {} },
    }, ...__VLS_functionalComponentArgsRest(__VLS_190));
    let __VLS_193;
    let __VLS_194;
    let __VLS_195;
    const __VLS_196 = {
        onClick: (__VLS_ctx.createAlert)
    };
    __VLS_192.slots.default;
    var __VLS_192;
}
var __VLS_152;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['max-w-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['pt-6']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:underline']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:underline']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:underline']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:underline']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['truncate']} */ ;
/** @type {__VLS_StyleScopedClasses['tabular']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['text-right']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-end']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            DIRECTION_OPTIONS: DIRECTION_OPTIONS,
            Card: Card,
            CardContent: CardContent,
            ProcessIcon: ProcessIcon,
            SpeedBadge: SpeedBadge,
            Input: Input,
            Label: Label,
            Select: Select,
            Switch: Switch,
            Button: Button,
            Badge: Badge,
            Table: Table,
            TableHeader: TableHeader,
            TableBody: TableBody,
            TableRow: TableRow,
            TableHead: TableHead,
            TableCell: TableCell,
            Dialog: Dialog,
            processStore: processStore,
            throttleOpen: throttleOpen,
            alertOpen: alertOpen,
            selected: selected,
            kbps: kbps,
            limitUpload: limitUpload,
            limitDownload: limitDownload,
            thresholdKb: thresholdKb,
            direction: direction,
            cooldown: cooldown,
            sorted: sorted,
            sortBy: sortBy,
            openThrottle: openThrottle,
            openAlert: openAlert,
            applyThrottle: applyThrottle,
            createAlert: createAlert,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
