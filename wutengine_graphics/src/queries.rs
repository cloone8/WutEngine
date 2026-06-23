//! GPU pipeline queries

use crate::label;

/// Helper struct for resolving various GPU queries
#[derive(Debug)]
pub struct QueryResolver {
    /// Buffers for timestamp queries
    timestamp_bufs: Option<(wgpu::QuerySet, wgpu::Buffer)>,

    /// Buffers for statistics queries
    statistics_bufs: Option<(wgpu::QuerySet, wgpu::Buffer)>,
}

impl QueryResolver {
    /// A new queryresolver on the given device, with the given debug name
    pub fn new(name: &str, device: &wgpu::Device) -> Self {
        let mut resolver = Self {
            timestamp_bufs: None,
            statistics_bufs: None,
        };

        let supported_features = crate::active_config().features;

        if supported_features.contains(wgpu::Features::TIMESTAMP_QUERY) {
            let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: label!("QueryResolver {} timestamp set", name),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            });

            let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: label!("QueryResolver {} timestamp resolve buffer", name),
                size: (wgpu::QUERY_SIZE * 2) as u64,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            resolver.timestamp_bufs = Some((query_set, query_buffer));
        }

        if supported_features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY) {
            let num_statistics = wgpu::PipelineStatisticsTypes::all().into_iter().count();

            let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: label!("QueryResolver {} statistics set", name),
                ty: wgpu::QueryType::PipelineStatistics(wgpu::PipelineStatisticsTypes::all()),
                count: num_statistics as u32,
            });

            let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: label!("QueryResolver {} statistics resolve buffer", name),
                size: (wgpu::QUERY_SIZE * num_statistics as u32) as u64,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            resolver.statistics_bufs = Some((query_set, query_buffer));
        }

        resolver
    }

    /// Returns the timestamp writes config for this query set
    pub fn renderpass_timestamp_writes(&mut self) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        let (query_set, _) = self.timestamp_bufs.as_ref()?;

        Some(wgpu::RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(0),
            end_of_pass_write_index: Some(1),
        })
    }

    /// Starts recording pipeline statistics on the given pass
    pub fn pipeline_statistics_start(&mut self, pass: &mut wgpu::RenderPass) {
        if let Some((statistics_set, _)) = self.statistics_bufs.as_ref() {
            pass.begin_pipeline_statistics_query(statistics_set, 0);
        }
    }

    /// Stops recording pipeline statistics on the given pass
    pub fn pipeline_statistics_end(&mut self, pass: &mut wgpu::RenderPass) {
        if self.statistics_bufs.as_ref().is_some() {
            pass.end_pipeline_statistics_query();
        }
    }

    /// Resolves the timestamp queries into a buffer, and copies the buffer's contents into the
    /// given target buffer at `offset_queries * wgpu::QUERY_SIZE` bytes from the start of the target buffer.
    /// Increments `offset_queries` by the amount of timestamp query indices written.
    pub fn resolve_timestamps(
        &self,
        cmd: &mut wgpu::CommandEncoder,
        target_buffer: &wgpu::Buffer,
        offset_queries: &mut u64,
    ) {
        let Some((timestamp_query_set, timestamp_query_buf)) = self.timestamp_bufs.as_ref() else {
            return;
        };

        debug_assert!(
            target_buffer.usage().contains(wgpu::BufferUsages::COPY_DST),
            "Cannot write timestamps to target buffer"
        );

        cmd.resolve_query_set(timestamp_query_set, 0..2, timestamp_query_buf, 0);
        cmd.copy_buffer_to_buffer(
            timestamp_query_buf,
            0,
            target_buffer,
            *offset_queries * (wgpu::QUERY_SIZE as u64),
            (wgpu::QUERY_SIZE * 2) as u64,
        );

        *offset_queries += 2;
    }

    /// Resolves the pipeline queries into a buffer, and copies the buffer's contents into the
    /// given target buffer at `offset_queries * wgpu::QUERY_SIZE` bytes from the start of the target buffer.
    /// Increments `offset_queries` by the amount of pipeline statistics query indices written.
    pub fn resolve_pipeline_statistics(
        &self,
        cmd: &mut wgpu::CommandEncoder,
        target_buffer: &wgpu::Buffer,
        offset_queries: &mut u64,
    ) {
        let Some((statistics_query_set, statistics_query_buf)) = self.statistics_bufs.as_ref()
        else {
            return;
        };

        debug_assert!(
            target_buffer.usage().contains(wgpu::BufferUsages::COPY_DST),
            "Cannot write statistics to target buffer"
        );

        let num_statistics = wgpu::PipelineStatisticsTypes::all().into_iter().count();

        cmd.resolve_query_set(statistics_query_set, 0..1, statistics_query_buf, 0);
        cmd.copy_buffer_to_buffer(
            statistics_query_buf,
            0,
            target_buffer,
            *offset_queries * (wgpu::QUERY_SIZE as u64),
            (wgpu::QUERY_SIZE * (num_statistics as u32)) as u64,
        );

        *offset_queries += num_statistics as u64;
    }
}
