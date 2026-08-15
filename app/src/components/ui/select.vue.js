import { computed } from 'vue';
import { cn } from '@/lib/utils';
const props = withDefaults(defineProps(), { options: () => [] });
const emit = defineEmits();
const normalized = computed(() => (props.options ?? []).map((o) => typeof o === 'string' ? { label: o, value: o } : o));
function onChange(e) {
    const el = e.target;
    const found = normalized.value.find((o) => String(o.value) === el.value);
    emit('update:modelValue', found ? found.value : el.value);
}
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_withDefaultsArg = (function (t) { return t; })({ options: () => [] });
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
__VLS_asFunctionalElement(__VLS_intrinsicElements.select, __VLS_intrinsicElements.select)({
    ...{ onChange: (__VLS_ctx.onChange) },
    value: (__VLS_ctx.modelValue),
    disabled: (__VLS_ctx.disabled),
    ...{ class: (__VLS_ctx.cn('flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50', props.class)) },
});
for (const [opt] of __VLS_getVForSourceType((__VLS_ctx.normalized))) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.option, __VLS_intrinsicElements.option)({
        key: (String(opt.value)),
        value: (opt.value),
    });
    (opt.label);
}
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            cn: cn,
            normalized: normalized,
            onChange: onChange,
        };
    },
    __typeEmits: {},
    __typeProps: {},
    props: {},
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
    __typeEmits: {},
    __typeProps: {},
    props: {},
});
; /* PartiallyEnd: #4569/main.vue */
