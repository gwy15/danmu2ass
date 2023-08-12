<template>
  <v-text-field
    v-bind="$attrs"
    v-model="displayValue"
    :label="$attrs.label"
    dense
    outlined
    hide-details
    type="number"
    @input="updateValue"
  ></v-text-field>
</template>
  
<script>
export default {
  name: "MyNumberField",

  inheritAttrs: false, // 禁用默认的属性继承

  data() {
    return {
      displayValue: null,
    };
  },

  methods: {
    updateValue() {
      this.$emit("input", parseFloat(this.displayValue));
    },
  },

  watch: {
    value(newValue) {
      this.displayValue = newValue;
    },
  },

  props: {
    value: {
      type: Number,
      required: true,
    },
  },

  mounted() {
    this.displayValue = this.value;
  },
};
</script>