<template>
  <v-text-field
    v-bind="$attrs"
    v-model="displayValue"
    :label="$attrs.label"
    suffix="%"
    dense
    outlined
    hide-details
    type="number"
    @input="updateValue"
  ></v-text-field>
</template>

<script>
export default {
  name: "MyPercentageField",

  inheritAttrs: false, // 禁用默认的属性继承

  data() {
    return {
      displayValue: null,
    };
  },

  methods: {
    updateValue() {
      this.$emit("input", parseFloat(this.displayValue) / 100);
    },
  },

  watch: {
    value(newValue) {
      this.displayValue = Math.floor(newValue * 100);
    },
  },

  props: {
    value: {
      type: Number,
      required: true,
    },
  },

  mounted() {
    this.displayValue = Math.floor(this.value * 100);
  },
};
</script>