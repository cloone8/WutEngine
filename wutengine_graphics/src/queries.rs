use core::num;

use smallvec::SmallVec;

#[derive(Debug)]
pub struct QueryResolver {
    timestamp_bufs: Option<(wgpu::QuerySet, wgpu::Buffer)>,
    statistics_bufs: Option<(wgpu::QuerySet, wgpu::Buffer)>,
    read_buffer: Option<wgpu::Buffer>,
}

impl QueryResolver {
    pub fn new(name: &str, device: &wgpu::Device) -> Self {
        let mut slots: u32 = 0;
        let mut resolver = Self {
            timestamp_bufs: None,
            statistics_bufs: None,
            read_buffer: None,
        };

        return resolver;

        let supported_features = crate::active_config().features;

        if supported_features.contains(wgpu::Features::TIMESTAMP_QUERY) {
            let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some(format!("QueryResolver {name} timestamp set").as_str()),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            });

            let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("QueryResolver {name} timestamp resolve buffer").as_str()),
                size: (wgpu::QUERY_SIZE * 2) as u64,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            resolver.timestamp_bufs = Some((query_set, query_buffer));

            slots += 2;
        }

        if supported_features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY) {
            let num_statistics = wgpu::PipelineStatisticsTypes::all().into_iter().count();

            let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some(format!("QueryResolver {name} statistics set").as_str()),
                ty: wgpu::QueryType::PipelineStatistics(wgpu::PipelineStatisticsTypes::all()),
                count: num_statistics as u32,
            });

            let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("QueryResolver {name} statistics resolve buffer").as_str()),
                size: (wgpu::QUERY_SIZE * num_statistics as u32) as u64,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            resolver.statistics_bufs = Some((query_set, query_buffer));

            slots += num_statistics as u32;
        }

        if slots != 0 {
            resolver.read_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(format!("QueryResolver {name} read buffer").as_str()),
                size: (wgpu::QUERY_SIZE * slots) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        resolver
    }

    pub fn renderpass_timestamp_writes(&mut self) -> Option<wgpu::RenderPassTimestampWrites> {
        let (query_set, _) = self.timestamp_bufs.as_ref()?;

        Some(wgpu::RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(0),
            end_of_pass_write_index: Some(1),
        })
    }

    pub fn pipeline_statistics_start(&mut self, pass: &mut wgpu::RenderPass) {
        if let Some((statistics_set, _)) = self.statistics_bufs.as_ref() {
            pass.begin_pipeline_statistics_query(statistics_set, 0);
        }
    }

    pub fn pipeline_statistics_end(&mut self, pass: &mut wgpu::RenderPass) {
        if self.statistics_bufs.as_ref().is_some() {
            pass.end_pipeline_statistics_query();
        }
    }

    pub fn resolve(&self, cmd: &mut wgpu::CommandEncoder) {
        let Some(read_buffer) = self.read_buffer.as_ref() else {
            return;
        };

        let mut num_slots = 0;

        let mut timestamps_resolved = false;
        if let Some((timestamp_query_set, timestamp_query_buf)) = self.timestamp_bufs.as_ref() {
            cmd.resolve_query_set(timestamp_query_set, 0..2, timestamp_query_buf, 0);
            cmd.copy_buffer_to_buffer(
                timestamp_query_buf,
                0,
                read_buffer,
                (num_slots * wgpu::QUERY_SIZE) as u64,
                (wgpu::QUERY_SIZE * 2) as u64,
            );

            timestamps_resolved = true;
            num_slots += 2;
        }

        let mut statistics_resolved = false;
        if let Some((statistics_query_set, statistics_query_buf)) = self.statistics_bufs.as_ref() {
            let num_statistics = wgpu::PipelineStatisticsTypes::all().into_iter().count();

            cmd.resolve_query_set(statistics_query_set, 0..1, statistics_query_buf, 0);
            cmd.copy_buffer_to_buffer(
                statistics_query_buf,
                0,
                read_buffer,
                (num_slots * wgpu::QUERY_SIZE) as u64,
                (wgpu::QUERY_SIZE * num_statistics as u32) as u64,
            );

            statistics_resolved = true;
            num_slots += num_statistics as u32;
        }

        if num_slots == 0 {
            return;
        }

        let read_buf_cpy = read_buffer.clone();

        cmd.map_buffer_on_submit(read_buffer, wgpu::MapMode::Read, .., move |result| {
            log::info!("In on-submit");
            if result.is_err() {
                read_buf_cpy.unmap();
                return;
            }

            let view = read_buf_cpy.get_mapped_range(..);

            let (view_slice, rest) = view.as_chunks::<{ wgpu::QUERY_SIZE as usize }>();

            assert_eq!(0, rest.len(), "Should not be left with rest");

            let mut data_offset = 0;
            if timestamps_resolved {
                let start = u64::from_ne_bytes(view_slice[data_offset]);
                let end = u64::from_ne_bytes(view_slice[data_offset + 1]);
                let duration = end.saturating_sub(start) as f32; // Timestamp may overflow in GPU
                let duration_nanos = duration * crate::queue().get_timestamp_period();

                log::info!("Duration nanos: {duration_nanos}");

                data_offset += 2;
            }

            if statistics_resolved {
                let num_statistics = wgpu::PipelineStatisticsTypes::all().into_iter().count();

                for (i, (statistic, _)) in wgpu::PipelineStatisticsTypes::all()
                    .iter_names()
                    .enumerate()
                {
                    let data = u64::from_ne_bytes(view_slice[data_offset + i]);

                    log::info!("{}: {}", statistic, data);
                }

                data_offset += num_statistics;
            }

            drop(view);
            read_buf_cpy.unmap();
        });
    }
}
