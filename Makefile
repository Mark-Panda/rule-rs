.PHONY: all test examples clean

# 默认目标
all: test examples

# 运行所有测试
test:
	cargo test --all

# 构建所有示例
build-examples:
	@for dir in examples/*; do \
		if [ -d "$$dir" ]; then \
			echo "Building $$dir..."; \
			(cd "$$dir" && cargo build) || exit 1; \
		fi \
	done

# 运行所有示例
examples: build-examples
	@for dir in examples/*; do \
		if [ -d "$$dir" ]; then \
			echo "Running $$(basename $$dir)..."; \
			(cd "$$dir" && cargo run --quiet) || exit 1; \
			echo ""; \
		fi \
	done

# 运行单个示例
example-%:
	@if [ -d "examples/$*" ]; then \
		echo "Running $*..."; \
		(cd "examples/$*" && cargo run) || exit 1; \
	else \
		echo "Example '$*' not found"; \
		exit 1; \
	fi

# 清理构建产物
clean:
	cargo clean
	@for dir in examples/*; do \
		if [ -d "$$dir" ]; then \
			(cd "$$dir" && cargo clean); \
		fi \
	done

# 可用的示例列表
list-examples:
	@echo "Available examples:"
	@for dir in examples/*; do \
		if [ -d "$$dir" ]; then \
			echo "  $$(basename $$dir)"; \
		fi \
	done

# 帮助信息
help:
	@echo "Available targets:"
	@echo "  all            - Run tests and examples"
	@echo "  test           - Run all tests"
	@echo "  examples       - Run all examples"
	@echo "  example-NAME   - Run specific example (e.g. make example-simple_rule)"
	@echo "  build-examples - Build all examples"
	@echo "  clean          - Clean build artifacts"
	@echo "  list-examples  - List available examples"
	@echo "  help           - Show this help message"

